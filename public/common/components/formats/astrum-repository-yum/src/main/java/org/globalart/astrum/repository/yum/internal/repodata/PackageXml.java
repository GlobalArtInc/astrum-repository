package org.globalart.astrum.repository.yum.internal.repodata;

import java.util.List;

import javax.annotation.Nullable;
import javax.xml.stream.XMLStreamException;
import javax.xml.stream.XMLStreamWriter;

import org.globalart.astrum.repository.yum.internal.rpm.RpmChangelog;
import org.globalart.astrum.repository.yum.internal.rpm.RpmDependency;
import org.globalart.astrum.repository.yum.internal.rpm.RpmFile;
import org.globalart.astrum.repository.yum.internal.rpm.RpmPackage;

import static org.globalart.astrum.repository.yum.internal.repodata.RepodataXml.COMMON_NS;
import static org.globalart.astrum.repository.yum.internal.repodata.RepodataXml.FILELISTS_NS;
import static org.globalart.astrum.repository.yum.internal.repodata.RepodataXml.OTHER_NS;
import static org.globalart.astrum.repository.yum.internal.repodata.RepodataXml.RPM_NS;
import static org.globalart.astrum.repository.yum.internal.repodata.RepodataXml.clean;

final class PackageXml
{
  private PackageXml() {
  }

  static void writePrimary(final XMLStreamWriter writer, final RepodataPackage repodataPackage)
      throws XMLStreamException
  {
    RpmPackage rpm = repodataPackage.rpm();
    writer.writeStartElement(COMMON_NS, "package");
    writer.writeAttribute("type", "rpm");
    text(writer, COMMON_NS, "name", rpm.name());
    text(writer, COMMON_NS, "arch", rpm.arch());
    version(writer, COMMON_NS, rpm);
    writer.writeStartElement(COMMON_NS, "checksum");
    writer.writeAttribute("type", "sha256");
    writer.writeAttribute("pkgid", "YES");
    writer.writeCharacters(repodataPackage.checksum());
    writer.writeEndElement();
    text(writer, COMMON_NS, "summary", rpm.summary());
    text(writer, COMMON_NS, "description", rpm.description());
    text(writer, COMMON_NS, "packager", rpm.packager());
    text(writer, COMMON_NS, "url", rpm.url());
    writer.writeEmptyElement(COMMON_NS, "time");
    writer.writeAttribute("file", Long.toString(repodataPackage.fileTime()));
    writer.writeAttribute("build", Long.toString(rpm.buildTime()));
    writer.writeEmptyElement(COMMON_NS, "size");
    writer.writeAttribute("package", Long.toString(repodataPackage.packageSize()));
    writer.writeAttribute("installed", Long.toString(rpm.installedSize()));
    writer.writeAttribute("archive", Long.toString(rpm.archiveSize()));
    writer.writeEmptyElement(COMMON_NS, "location");
    writer.writeAttribute("href", clean(repodataPackage.location()));
    writer.writeStartElement(COMMON_NS, "format");
    text(writer, RPM_NS, "license", rpm.license());
    text(writer, RPM_NS, "vendor", rpm.vendor());
    text(writer, RPM_NS, "group", rpm.group());
    text(writer, RPM_NS, "buildhost", rpm.buildHost());
    text(writer, RPM_NS, "sourcerpm", rpm.sourceRpm());
    writer.writeEmptyElement(RPM_NS, "header-range");
    writer.writeAttribute("start", Long.toString(rpm.headerStart()));
    writer.writeAttribute("end", Long.toString(rpm.headerEnd()));
    dependencies(writer, "provides", rpm.provides());
    dependencies(writer, "requires", rpm.requires());
    dependencies(writer, "conflicts", rpm.conflicts());
    dependencies(writer, "obsoletes", rpm.obsoletes());
    dependencies(writer, "suggests", rpm.suggests());
    dependencies(writer, "enhances", rpm.enhances());
    dependencies(writer, "recommends", rpm.recommends());
    dependencies(writer, "supplements", rpm.supplements());
    for (RpmFile file : rpm.files()) {
      if (file.isPrimary()) {
        file(writer, COMMON_NS, file);
      }
    }
    writer.writeEndElement();
    writer.writeEndElement();
  }

  static void writeFilelists(final XMLStreamWriter writer, final RepodataPackage repodataPackage)
      throws XMLStreamException
  {
    RpmPackage rpm = repodataPackage.rpm();
    start(writer, FILELISTS_NS, repodataPackage);
    for (RpmFile file : rpm.files()) {
      file(writer, FILELISTS_NS, file);
    }
    writer.writeEndElement();
  }

  static void writeOther(final XMLStreamWriter writer, final RepodataPackage repodataPackage)
      throws XMLStreamException
  {
    RpmPackage rpm = repodataPackage.rpm();
    start(writer, OTHER_NS, repodataPackage);
    for (RpmChangelog changelog : rpm.changelogs()) {
      writer.writeStartElement(OTHER_NS, "changelog");
      writer.writeAttribute("author", clean(changelog.author()));
      writer.writeAttribute("date", Long.toString(changelog.date()));
      writer.writeCharacters(clean(changelog.text()));
      writer.writeEndElement();
    }
    writer.writeEndElement();
  }

  private static void start(final XMLStreamWriter writer, final String namespace, final RepodataPackage repodataPackage)
      throws XMLStreamException
  {
    RpmPackage rpm = repodataPackage.rpm();
    writer.writeStartElement(namespace, "package");
    writer.writeAttribute("pkgid", repodataPackage.checksum());
    writer.writeAttribute("name", clean(rpm.name()));
    writer.writeAttribute("arch", clean(rpm.arch()));
    version(writer, namespace, rpm);
  }

  private static void version(final XMLStreamWriter writer, final String namespace, final RpmPackage rpm)
      throws XMLStreamException
  {
    writer.writeEmptyElement(namespace, "version");
    writer.writeAttribute("epoch", clean(rpm.epoch()));
    writer.writeAttribute("ver", clean(rpm.version()));
    writer.writeAttribute("rel", clean(rpm.release()));
  }

  private static void dependencies(final XMLStreamWriter writer, final String name, final List<RpmDependency> entries)
      throws XMLStreamException
  {
    if (entries.isEmpty()) {
      return;
    }
    writer.writeStartElement(RPM_NS, name);
    for (RpmDependency entry : entries) {
      writer.writeEmptyElement(RPM_NS, "entry");
      writer.writeAttribute("name", clean(entry.name()));
      if (entry.flags() != null) {
        writer.writeAttribute("flags", entry.flags());
        attribute(writer, "epoch", entry.epoch());
        attribute(writer, "ver", entry.version());
        attribute(writer, "rel", entry.release());
      }
      if (entry.pre()) {
        writer.writeAttribute("pre", "1");
      }
    }
    writer.writeEndElement();
  }

  private static void file(final XMLStreamWriter writer, final String namespace, final RpmFile file)
      throws XMLStreamException
  {
    writer.writeStartElement(namespace, "file");
    if (!file.type().isEmpty()) {
      writer.writeAttribute("type", file.type());
    }
    writer.writeCharacters(clean(file.path()));
    writer.writeEndElement();
  }

  private static void attribute(final XMLStreamWriter writer, final String name, @Nullable final String value)
      throws XMLStreamException
  {
    if (value != null) {
      writer.writeAttribute(name, clean(value));
    }
  }

  private static void text(
      final XMLStreamWriter writer,
      final String namespace,
      final String name,
      @Nullable final String value) throws XMLStreamException
  {
    if (value == null || value.isEmpty()) {
      writer.writeEmptyElement(namespace, name);
      return;
    }
    writer.writeStartElement(namespace, name);
    writer.writeCharacters(clean(value));
    writer.writeEndElement();
  }
}
