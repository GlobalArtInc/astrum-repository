package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.BufferedOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Map.Entry;
import java.util.zip.GZIPOutputStream;

import javax.xml.stream.XMLStreamException;
import javax.xml.stream.XMLStreamReader;
import javax.xml.stream.XMLStreamWriter;

public final class MetadataFileWriter
    implements AutoCloseable
{
  private final MetadataType type;

  private final Path file;

  private final DigestingOutputStream compressed;

  private final DigestingOutputStream open;

  private final OutputStream gzip;

  private final XMLStreamWriter writer;

  private boolean finished;

  public MetadataFileWriter(final MetadataType type, final long packages) throws IOException {
    this.type = type;
    this.file = Files.createTempFile("yum-" + type.type(), ".xml.gz");
    boolean opened = false;
    try {
      compressed = new DigestingOutputStream(new BufferedOutputStream(Files.newOutputStream(file), 65536));
      gzip = new GZIPOutputStream(compressed, 65536);
      open = new DigestingOutputStream(gzip);
      writer = RepodataXml.writer(new BufferedOutputStream(open, 65536));
      writer.setDefaultNamespace(type.namespace());
      writer.writeStartElement("", type.rootElement(), type.namespace());
      for (Entry<String, String> prefix : type.prefixes().entrySet()) {
        writer.setPrefix(prefix.getKey(), prefix.getValue());
        writer.writeNamespace(prefix.getKey(), prefix.getValue());
      }
      writer.writeAttribute("packages", Long.toString(packages));
      writer.writeCharacters("\n");
      opened = true;
    }
    catch (XMLStreamException e) {
      throw new IOException(e);
    }
    finally {
      if (!opened) {
        Files.deleteIfExists(file);
      }
    }
  }

  public XMLStreamWriter writer() {
    return writer;
  }

  public void write(final RepodataPackage repodataPackage) throws IOException {
    try {
      switch (type) {
        case PRIMARY -> PackageXml.writePrimary(writer, repodataPackage);
        case FILELISTS -> PackageXml.writeFilelists(writer, repodataPackage);
        case OTHER -> PackageXml.writeOther(writer, repodataPackage);
      }
      writer.writeCharacters("\n");
    }
    catch (XMLStreamException e) {
      throw new IOException(e);
    }
  }

  public void copy(final XMLStreamReader reader) throws IOException {
    try {
      RepodataXml.copyElement(reader, writer);
      writer.writeCharacters("\n");
    }
    catch (XMLStreamException e) {
      throw new IOException(e);
    }
  }

  public GeneratedMetadata finish(final long timestamp) throws IOException {
    try {
      writer.writeEndElement();
      writer.writeCharacters("\n");
      writer.flush();
      writer.close();
    }
    catch (XMLStreamException e) {
      throw new IOException(e);
    }
    open.flush();
    gzip.close();
    finished = true;
    String checksum = compressed.hex();
    RepomdData data = new RepomdData(type.type(), "sha256", checksum, "sha256", open.hex(),
        "repodata/" + type.fileName(checksum), timestamp, compressed.count(), open.count(), null);
    return new GeneratedMetadata(type, file, data);
  }

  @Override
  public void close() throws IOException {
    if (!finished) {
      try {
        gzip.close();
      }
      finally {
        Files.deleteIfExists(file);
      }
    }
  }
}
