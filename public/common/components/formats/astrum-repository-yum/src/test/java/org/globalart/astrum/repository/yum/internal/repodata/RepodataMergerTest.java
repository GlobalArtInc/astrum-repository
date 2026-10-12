package org.globalart.astrum.repository.yum.internal.repodata;

import java.io.ByteArrayOutputStream;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.yum.internal.rpm.TestRpm;

import org.junit.Test;
import org.tukaani.xz.LZMA2Options;
import org.tukaani.xz.XZOutputStream;
import org.w3c.dom.Element;
import org.w3c.dom.NodeList;

import static org.globalart.astrum.repository.yum.internal.repodata.RepodataBuilderTest.document;
import static org.junit.Assert.assertEquals;

public class RepodataMergerTest
{
  @Test
  public void mergesMembersAndKeepsTheFirstPackageAtEachLocation() throws Exception {
    TestPackageSource shared = new TestPackageSource("Packages/shared-1.0-1.x86_64.rpm",
        new TestRpm("shared", "1.0", "1").file("/usr/bin/shared", 1).build(), 1L);
    TestPackageSource sharedRebuilt = new TestPackageSource("Packages/shared-1.0-1.x86_64.rpm",
        new TestRpm("shared", "1.0", "1").buildTime(1800000000L).file("/usr/bin/shared", 2).build(), 1L);
    TestPackageSource first = new TestPackageSource("Packages/first-1.0-1.x86_64.rpm",
        new TestRpm("first", "1.0", "1").file("/usr/bin/first", 1).build(), 1L);
    TestPackageSource second = new TestPackageSource("Packages/second-2.0-1.noarch.rpm",
        new TestRpm("second", "2.0", "1").arch("noarch").changelog(5L, "Me", "- Second").build(), 1L);

    try (Repodata one = RepodataBuilder.build(List.of(shared, first), 10L);
         Repodata two = RepodataBuilder.build(List.of(sharedRebuilt, second), 20L);
         Repodata merged = RepodataMerger.merge(List.of(member("one", one), member("two", two)), 30L))
    {
      assertEquals(3, merged.packages());
      Element primary = document(merged, MetadataType.PRIMARY).getDocumentElement();
      assertEquals("3", primary.getAttribute("packages"));
      assertEquals(List.of("shared", "first", "second"), names(primary, RepodataXml.COMMON_NS));
      assertEquals(List.of(shared.checksum(), first.checksum(), second.checksum()),
          texts(primary.getElementsByTagNameNS(RepodataXml.COMMON_NS, "checksum")));
      assertEquals(3, primary.getElementsByTagNameNS(RepodataXml.RPM_NS, "provides").getLength());

      Element filelists = document(merged, MetadataType.FILELISTS).getDocumentElement();
      assertEquals(List.of(shared.checksum(), first.checksum(), second.checksum()),
          attributes(filelists.getElementsByTagNameNS(RepodataXml.FILELISTS_NS, "package"), "pkgid"));
      Element other = document(merged, MetadataType.OTHER).getDocumentElement();
      assertEquals(List.of(shared.checksum(), first.checksum(), second.checksum()),
          attributes(other.getElementsByTagNameNS(RepodataXml.OTHER_NS, "package"), "pkgid"));
      assertEquals("- Second",
          other.getElementsByTagNameNS(RepodataXml.OTHER_NS, "changelog").item(0).getTextContent());
    }
  }

  @Test
  public void copiesPackagesWrittenWithOtherPrefixesAndCompressions() throws Exception {
    String primary = "<?xml version=\"1.0\"?>\n"
        + "<common:metadata xmlns:common=\"" + RepodataXml.COMMON_NS + "\" xmlns:r=\"" + RepodataXml.RPM_NS
        + "\" packages=\"1\">\n"
        + "<common:package type=\"rpm\"><common:name>foreign</common:name><common:arch>noarch</common:arch>"
        + "<common:version epoch=\"0\" ver=\"1\" rel=\"1\"/>"
        + "<common:checksum type=\"sha256\" pkgid=\"YES\">abc</common:checksum>"
        + "<common:location xml:base=\"https://mirror.example.com/os/\" href=\"Packages/foreign-1-1.noarch.rpm\"/>"
        + "<common:format><r:license>MIT</r:license><r:provides><r:entry name=\"foreign\"/></r:provides>"
        + "</common:format></common:package>\n"
        + "</common:metadata>\n";
    String filelists = "<filelists xmlns=\"" + RepodataXml.FILELISTS_NS + "\" packages=\"1\">"
        + "<package pkgid=\"abc\" name=\"foreign\" arch=\"noarch\"><version epoch=\"0\" ver=\"1\" rel=\"1\"/>"
        + "<file>/usr/bin/foreign</file></package></filelists>";
    Map<String, byte[]> files = new HashMap<>();
    files.put("repodata/primary.xml.xz", xz(primary.getBytes(StandardCharsets.UTF_8)));
    files.put("repodata/filelists.xml", filelists.getBytes(StandardCharsets.UTF_8));
    Repomd repomd = new Repomd("1", List.of(
        new RepomdData("primary", "sha256", "x", null, null, "repodata/primary.xml.xz", 1L, null, null, null),
        new RepomdData("filelists", "sha256", "y", null, null, "repodata/filelists.xml", 1L, null, null, null)));
    MemberRepodata member = new MemberRepodata()
    {
      @Override
      public String name() {
        return "foreign";
      }

      @Override
      public Repomd repomd() {
        return repomd;
      }

      @Override
      public InputStream open(final RepomdData data) throws IOException {
        return Decompression.open(new ByteArrayInputStream(files.get(data.location())));
      }
    };

    try (Repodata merged = RepodataMerger.merge(List.of(member), 2L)) {
      Element root = document(merged, MetadataType.PRIMARY).getDocumentElement();
      Element location = (Element) root.getElementsByTagNameNS(RepodataXml.COMMON_NS, "location").item(0);
      assertEquals("https://mirror.example.com/os/", location.getAttributeNS(RepodataXml.XML_NS, "base"));
      assertEquals("MIT", root.getElementsByTagNameNS(RepodataXml.RPM_NS, "license").item(0).getTextContent());
      Element filelistsRoot = document(merged, MetadataType.FILELISTS).getDocumentElement();
      assertEquals("/usr/bin/foreign",
          filelistsRoot.getElementsByTagNameNS(RepodataXml.FILELISTS_NS, "file").item(0).getTextContent());
      Element otherRoot = document(merged, MetadataType.OTHER).getDocumentElement();
      assertEquals(0, otherRoot.getElementsByTagNameNS(RepodataXml.OTHER_NS, "package").getLength());
    }
  }

  private static byte[] xz(final byte[] bytes) throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (XZOutputStream xz = new XZOutputStream(out, new LZMA2Options())) {
      xz.write(bytes);
    }
    return out.toByteArray();
  }

  static MemberRepodata member(final String name, final Repodata repodata) {
    return new MemberRepodata()
    {
      @Override
      public String name() {
        return name;
      }

      @Override
      public Repomd repomd() {
        return repodata.repomd();
      }

      @Override
      public InputStream open(final RepomdData data) throws IOException {
        GeneratedMetadata file = repodata.files().stream()
            .filter(candidate -> candidate.data().equals(data))
            .findFirst()
            .orElseThrow();
        return Decompression.open(Files.newInputStream(file.file()));
      }
    };
  }

  private static List<String> names(final Element root, final String namespace) {
    List<String> names = new ArrayList<>();
    NodeList packages = root.getElementsByTagNameNS(namespace, "package");
    for (int i = 0; i < packages.getLength(); i++) {
      names.add(((Element) packages.item(i)).getElementsByTagNameNS(namespace, "name").item(0).getTextContent());
    }
    return names;
  }

  private static List<String> texts(final NodeList nodes) {
    List<String> texts = new ArrayList<>();
    for (int i = 0; i < nodes.getLength(); i++) {
      texts.add(nodes.item(i).getTextContent());
    }
    return texts;
  }

  private static List<String> attributes(final NodeList nodes, final String attribute) {
    List<String> values = new ArrayList<>();
    for (int i = 0; i < nodes.getLength(); i++) {
      values.add(((Element) nodes.item(i)).getAttribute(attribute));
    }
    return values;
  }
}
