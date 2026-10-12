package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.List;
import java.util.Optional;

import javax.annotation.Nullable;
import javax.xml.stream.XMLStreamConstants;
import javax.xml.stream.XMLStreamException;
import javax.xml.stream.XMLStreamReader;
import javax.xml.stream.XMLStreamWriter;

public record Repomd(@Nullable String revision, List<RepomdData> data)
{
  public static final String FILE_NAME = "repomd.xml";

  public Optional<RepomdData> data(final String type) {
    return data.stream().filter(entry -> entry.type().equals(type)).findFirst();
  }

  public static Repomd parse(final InputStream in) throws IOException {
    try {
      XMLStreamReader reader = RepodataXml.reader(in);
      try {
        if (!RepodataXml.nextElement(reader) || !"repomd".equals(reader.getLocalName())) {
          throw new IOException("Not a repomd.xml document");
        }
        String revision = null;
        List<RepomdData> data = new ArrayList<>();
        while (reader.hasNext()) {
          int event = reader.next();
          if (event == XMLStreamConstants.END_ELEMENT && "repomd".equals(reader.getLocalName())) {
            break;
          }
          if (event != XMLStreamConstants.START_ELEMENT) {
            continue;
          }
          if ("revision".equals(reader.getLocalName())) {
            revision = reader.getElementText().trim();
          }
          else if ("data".equals(reader.getLocalName())) {
            data.add(readData(reader));
          }
          else {
            RepodataXml.skipElement(reader);
          }
        }
        return new Repomd(revision, List.copyOf(data));
      }
      finally {
        reader.close();
      }
    }
    catch (XMLStreamException e) {
      throw new IOException("Unreadable repomd.xml: " + e.getMessage(), e);
    }
  }

  private static RepomdData readData(final XMLStreamReader reader) throws XMLStreamException, IOException {
    String type = reader.getAttributeValue(null, "type");
    String checksumType = null;
    String checksum = null;
    String openChecksumType = null;
    String openChecksum = null;
    String location = null;
    String locationBase = null;
    Long timestamp = null;
    Long size = null;
    Long openSize = null;
    while (reader.hasNext()) {
      int event = reader.next();
      if (event == XMLStreamConstants.END_ELEMENT && "data".equals(reader.getLocalName())) {
        break;
      }
      if (event != XMLStreamConstants.START_ELEMENT) {
        continue;
      }
      switch (reader.getLocalName()) {
        case "checksum" -> {
          checksumType = reader.getAttributeValue(null, "type");
          checksum = reader.getElementText().trim();
        }
        case "open-checksum" -> {
          openChecksumType = reader.getAttributeValue(null, "type");
          openChecksum = reader.getElementText().trim();
        }
        case "location" -> {
          location = reader.getAttributeValue(null, "href");
          locationBase = reader.getAttributeValue(RepodataXml.XML_NS, "base");
          RepodataXml.skipElement(reader);
        }
        case "timestamp" -> timestamp = number(reader.getElementText());
        case "size" -> size = number(reader.getElementText());
        case "open-size" -> openSize = number(reader.getElementText());
        default -> RepodataXml.skipElement(reader);
      }
    }
    if (type == null || location == null || checksum == null) {
      throw new IOException("repomd.xml has an incomplete data entry");
    }
    return new RepomdData(type, checksumType == null ? "sha256" : checksumType, checksum, openChecksumType,
        openChecksum, location, timestamp, size, openSize, locationBase);
  }

  @Nullable
  private static Long number(final String text) {
    try {
      return Long.parseLong(text.trim());
    }
    catch (NumberFormatException e) {
      return null;
    }
  }

  public byte[] toXml() throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try {
      XMLStreamWriter writer = RepodataXml.writer(out);
      writer.setDefaultNamespace(RepodataXml.REPO_NS);
      writer.writeStartElement("", "repomd", RepodataXml.REPO_NS);
      writer.setPrefix("rpm", RepodataXml.RPM_NS);
      writer.writeNamespace("rpm", RepodataXml.RPM_NS);
      if (revision != null) {
        writer.writeCharacters("\n  ");
        element(writer, "revision", revision);
      }
      for (RepomdData entry : data) {
        writer.writeCharacters("\n  ");
        writer.writeStartElement(RepodataXml.REPO_NS, "data");
        writer.writeAttribute("type", entry.type());
        writer.writeCharacters("\n    ");
        writer.writeStartElement(RepodataXml.REPO_NS, "checksum");
        writer.writeAttribute("type", entry.checksumType());
        writer.writeCharacters(entry.checksum());
        writer.writeEndElement();
        if (entry.openChecksum() != null) {
          writer.writeCharacters("\n    ");
          writer.writeStartElement(RepodataXml.REPO_NS, "open-checksum");
          writer.writeAttribute("type", entry.openChecksumType() == null ? "sha256" : entry.openChecksumType());
          writer.writeCharacters(entry.openChecksum());
          writer.writeEndElement();
        }
        writer.writeCharacters("\n    ");
        writer.writeEmptyElement(RepodataXml.REPO_NS, "location");
        if (entry.locationBase() != null) {
          writer.writeAttribute("xml", RepodataXml.XML_NS, "base", entry.locationBase());
        }
        writer.writeAttribute("href", entry.location());
        if (entry.timestamp() != null) {
          writer.writeCharacters("\n    ");
          element(writer, "timestamp", entry.timestamp().toString());
        }
        if (entry.size() != null) {
          writer.writeCharacters("\n    ");
          element(writer, "size", entry.size().toString());
        }
        if (entry.openSize() != null) {
          writer.writeCharacters("\n    ");
          element(writer, "open-size", entry.openSize().toString());
        }
        writer.writeCharacters("\n  ");
        writer.writeEndElement();
      }
      writer.writeCharacters("\n");
      writer.writeEndElement();
      writer.writeCharacters("\n");
      writer.close();
    }
    catch (XMLStreamException e) {
      throw new IOException(e);
    }
    return out.toByteArray();
  }

  private static void element(final XMLStreamWriter writer, final String name, final String value)
      throws XMLStreamException
  {
    writer.writeStartElement(RepodataXml.REPO_NS, name);
    writer.writeCharacters(value);
    writer.writeEndElement();
  }
}
