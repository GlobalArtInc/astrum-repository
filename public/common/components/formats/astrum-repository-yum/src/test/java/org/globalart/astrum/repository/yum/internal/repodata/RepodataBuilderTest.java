package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.ByteArrayInputStream;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.security.MessageDigest;
import java.util.HexFormat;
import java.util.List;
import java.util.zip.GZIPInputStream;

import javax.xml.parsers.DocumentBuilderFactory;

import org.globalart.astrum.repository.yum.internal.rpm.TestRpm;

import org.junit.Test;
import org.w3c.dom.Document;
import org.w3c.dom.Element;
import org.w3c.dom.NodeList;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;

public class RepodataBuilderTest
{
  @Test
  public void writesPrimaryFilelistsAndOtherWithChecksums() throws Exception {
    TestPackageSource hello = new TestPackageSource("Packages/h/hello-2.12.1-3.el9.x86_64.rpm",
        new TestRpm("hello", "2.12.1", "3.el9")
            .summary("Prints <greetings> & more")
            .url("https://www.gnu.org/software/hello/")
            .directory("/usr/share/hello")
            .file("/usr/bin/hello", 1024)
            .file("/usr/share/hello/README", 100)
            .ghost("/var/log/hello.log")
            .requires("glibc", 8 | 4, "2.34")
            .requires("/bin/sh", 1 << 9, "")
            .changelog(1712300000L, "Jane Doe <jane@example.com> - 2.12.1-3", "- Rebuild")
            .build(), 1712400000L);
    TestPackageSource tool = new TestPackageSource("Packages/t/tool-1.0-1.noarch.rpm",
        new TestRpm("tool", "1.0", "1").arch("noarch").epoch(2).file("/etc/tool.conf", 5).build(), 1712400001L);

    try (Repodata repodata = RepodataBuilder.build(List.of(hello, tool), 1712500000L)) {
      assertEquals(2, repodata.packages());
      assertEquals("1712500000", repodata.repomd().revision());
      assertEquals(List.of("primary", "filelists", "other"),
          repodata.repomd().data().stream().map(RepomdData::type).toList());

      for (GeneratedMetadata file : repodata.files()) {
        byte[] compressed = Files.readAllBytes(file.file());
        byte[] open = new GZIPInputStream(new ByteArrayInputStream(compressed)).readAllBytes();
        RepomdData data = file.data();
        assertEquals(sha256(compressed), data.checksum());
        assertEquals(sha256(open), data.openChecksum());
        assertEquals(compressed.length, (long) data.size());
        assertEquals(open.length, (long) data.openSize());
        assertEquals("repodata/" + data.checksum() + "-" + data.type() + ".xml.gz", data.location());
        assertEquals(1712500000L, (long) data.timestamp());
      }

      Document primary = document(repodata, MetadataType.PRIMARY);
      Element root = primary.getDocumentElement();
      assertEquals(RepodataXml.COMMON_NS, root.getNamespaceURI());
      assertEquals("metadata", root.getLocalName());
      assertEquals("2", root.getAttribute("packages"));
      NodeList packages = root.getElementsByTagNameNS(RepodataXml.COMMON_NS, "package");
      assertEquals(2, packages.getLength());
      Element first = (Element) packages.item(0);
      assertEquals("hello", text(first, RepodataXml.COMMON_NS, "name"));
      assertEquals("x86_64", text(first, RepodataXml.COMMON_NS, "arch"));
      assertEquals("Prints <greetings> & more", text(first, RepodataXml.COMMON_NS, "summary"));
      assertEquals(hello.checksum(), text(first, RepodataXml.COMMON_NS, "checksum"));
      Element version = child(first, RepodataXml.COMMON_NS, "version");
      assertEquals("0", version.getAttribute("epoch"));
      assertEquals("2.12.1", version.getAttribute("ver"));
      assertEquals("3.el9", version.getAttribute("rel"));
      assertEquals("Packages/h/hello-2.12.1-3.el9.x86_64.rpm",
          child(first, RepodataXml.COMMON_NS, "location").getAttribute("href"));
      assertEquals(Long.toString(hello.size()), child(first, RepodataXml.COMMON_NS, "size").getAttribute("package"));
      assertEquals("1712400000", child(first, RepodataXml.COMMON_NS, "time").getAttribute("file"));
      NodeList requires = ((Element) first.getElementsByTagNameNS(RepodataXml.RPM_NS, "requires").item(0))
          .getElementsByTagNameNS(RepodataXml.RPM_NS, "entry");
      assertEquals(2, requires.getLength());
      assertEquals("GE", ((Element) requires.item(0)).getAttribute("flags"));
      assertEquals("1", ((Element) requires.item(1)).getAttribute("pre"));
      NodeList primaryFiles = first.getElementsByTagNameNS(RepodataXml.COMMON_NS, "file");
      assertEquals(1, primaryFiles.getLength());
      assertEquals("/usr/bin/hello", primaryFiles.item(0).getTextContent());
      Element second = (Element) packages.item(1);
      assertEquals("2", child(second, RepodataXml.COMMON_NS, "version").getAttribute("epoch"));

      Document filelists = document(repodata, MetadataType.FILELISTS);
      Element filelistsPackage = (Element) filelists.getDocumentElement()
          .getElementsByTagNameNS(RepodataXml.FILELISTS_NS, "package").item(0);
      assertEquals(hello.checksum(), filelistsPackage.getAttribute("pkgid"));
      NodeList files = filelistsPackage.getElementsByTagNameNS(RepodataXml.FILELISTS_NS, "file");
      assertEquals(4, files.getLength());
      assertEquals("dir", ((Element) files.item(0)).getAttribute("type"));
      assertEquals("ghost", ((Element) files.item(3)).getAttribute("type"));

      Document other = document(repodata, MetadataType.OTHER);
      Element changelog = (Element) other.getDocumentElement()
          .getElementsByTagNameNS(RepodataXml.OTHER_NS, "changelog").item(0);
      assertEquals("Jane Doe <jane@example.com> - 2.12.1-3", changelog.getAttribute("author"));
      assertEquals("1712300000", changelog.getAttribute("date"));
      assertEquals("- Rebuild", changelog.getTextContent());

      Repomd parsed = Repomd.parse(new ByteArrayInputStream(repodata.repomd().toXml()));
      assertEquals(repodata.repomd(), parsed);
    }
  }

  @Test
  public void writesEmptyRepodata() throws Exception {
    try (Repodata repodata = RepodataBuilder.build(List.of(), 1L)) {
      assertEquals(0, repodata.packages());
      assertEquals("0", document(repodata, MetadataType.PRIMARY).getDocumentElement().getAttribute("packages"));
    }
  }

  @Test
  public void leavesUnreadablePackagesOut() throws Exception {
    TestPackageSource good = new TestPackageSource("good-1-1.x86_64.rpm", new TestRpm("good", "1", "1").build(), 1L);
    TestPackageSource bad = new TestPackageSource("bad-1-1.x86_64.rpm", "garbage".getBytes(StandardCharsets.UTF_8),
        1L);

    try (Repodata repodata = RepodataBuilder.build(List.of(bad, good), 1L)) {
      assertEquals(1, repodata.packages());
      assertEquals(List.of("bad-1-1.x86_64.rpm"), repodata.skipped());
      Element root = document(repodata, MetadataType.PRIMARY).getDocumentElement();
      assertEquals("1", root.getAttribute("packages"));
      assertEquals(1, root.getElementsByTagNameNS(RepodataXml.COMMON_NS, "package").getLength());
    }
  }

  @Test
  public void stripsCharactersXmlCannotCarry() throws Exception {
    TestPackageSource odd = new TestPackageSource("odd-1-1.x86_64.rpm",
        new TestRpm("odd", "1", "1").description("bell\u0007 and tab\t").build(), 1L);

    try (Repodata repodata = RepodataBuilder.build(List.of(odd), 1L)) {
      Element root = document(repodata, MetadataType.PRIMARY).getDocumentElement();
      assertEquals("bell and tab\t", text(root, RepodataXml.COMMON_NS, "description"));
      String xml = new String(open(repodata, MetadataType.PRIMARY), StandardCharsets.UTF_8);
      assertTrue(xml, xml.startsWith("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<metadata xmlns=\""
          + RepodataXml.COMMON_NS + "\" xmlns:rpm=\"" + RepodataXml.RPM_NS + "\" packages=\"1\">\n<package type=\"rpm\">"));
    }
  }

  static Document document(final Repodata repodata, final MetadataType type) throws Exception {
    return parse(open(repodata, type));
  }

  static Document parse(final byte[] xml) throws Exception {
    DocumentBuilderFactory factory = DocumentBuilderFactory.newInstance();
    factory.setNamespaceAware(true);
    try (InputStream in = new ByteArrayInputStream(xml)) {
      return factory.newDocumentBuilder().parse(in);
    }
  }

  static byte[] open(final Repodata repodata, final MetadataType type) throws Exception {
    GeneratedMetadata file = repodata.files().stream().filter(f -> f.type() == type).findFirst().orElseThrow();
    try (InputStream in = new GZIPInputStream(Files.newInputStream(file.file()))) {
      return in.readAllBytes();
    }
  }

  static String text(final Element parent, final String namespace, final String name) {
    return child(parent, namespace, name).getTextContent();
  }

  static Element child(final Element parent, final String namespace, final String name) {
    return (Element) parent.getElementsByTagNameNS(namespace, name).item(0);
  }

  static String sha256(final byte[] bytes) throws Exception {
    return HexFormat.of().formatHex(MessageDigest.getInstance("SHA-256").digest(bytes));
  }
}
