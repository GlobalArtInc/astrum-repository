package org.globalart.astrum.repository.yum.internal.rpm;

import java.io.ByteArrayInputStream;
import java.util.Arrays;
import java.util.List;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertThrows;
import static org.junit.Assert.assertTrue;

public class RpmParserTest
{
  @Test
  public void readsTheHeaderOfABinaryPackage() throws Exception {
    byte[] rpm = new TestRpm("hello", "2.12.1", "3.el9")
        .epoch(1)
        .summary("Prints a familiar, friendly greeting")
        .description("Hello prints a friendly greeting.\nIt is an example of a GNU package.")
        .license("GPL-3.0-or-later")
        .url("https://www.gnu.org/software/hello/")
        .packager("Build System <build@example.com>")
        .vendor("Example")
        .buildTime(1712345678L)
        .directory("/usr/share/hello")
        .file("/usr/bin/hello", 1024)
        .file("/usr/share/hello/README", 100)
        .file("/etc/hello.conf", 10)
        .ghost("/var/log/hello.log")
        .provides("greeting", 8 | 4, "2.0")
        .requires("glibc", 8 | 4, "2.34")
        .requires("/bin/sh", (1 << 9) | (1 << 6), "")
        .requires("/usr/bin/hello", 0, "")
        .requires("glibc", 8 | 4, "2.34")
        .conflicts("hello-legacy", 2, "2.0")
        .obsoletes("hi", 2 | 8, "1:1.0-1")
        .recommends("hello-doc", 0, "")
        .changelog(1712300000L, "Jane Doe <jane@example.com> - 1:2.12.1-3", "- Rebuild")
        .changelog(1712200000L, "Jane Doe <jane@example.com> - 1:2.12.1-2", "- Fix <build> & tests")
        .build();

    RpmPackage rpmPackage = RpmParser.parse(new ByteArrayInputStream(rpm));

    assertEquals("hello", rpmPackage.name());
    assertEquals("1", rpmPackage.epoch());
    assertEquals("2.12.1", rpmPackage.version());
    assertEquals("3.el9", rpmPackage.release());
    assertEquals("x86_64", rpmPackage.arch());
    assertEquals("1:2.12.1-3.el9", rpmPackage.componentVersion());
    assertEquals("Prints a familiar, friendly greeting", rpmPackage.summary());
    assertEquals("Hello prints a friendly greeting.\nIt is an example of a GNU package.", rpmPackage.description());
    assertEquals("GPL-3.0-or-later", rpmPackage.license());
    assertEquals("https://www.gnu.org/software/hello/", rpmPackage.url());
    assertEquals("Build System <build@example.com>", rpmPackage.packager());
    assertEquals("Example", rpmPackage.vendor());
    assertEquals("Unspecified", rpmPackage.group());
    assertEquals("build.example.com", rpmPackage.buildHost());
    assertEquals("hello-2.12.1-3.el9.src.rpm", rpmPackage.sourceRpm());
    assertEquals(1712345678L, rpmPackage.buildTime());
    assertEquals(1134L, rpmPackage.installedSize());
    assertTrue(rpmPackage.archiveSize() > 0);
    assertFalse(rpmPackage.isSource());

    assertEquals(List.of(
        new RpmFile("/usr/share/hello", RpmFile.DIR),
        new RpmFile("/usr/bin/hello", RpmFile.FILE),
        new RpmFile("/usr/share/hello/README", RpmFile.FILE),
        new RpmFile("/etc/hello.conf", RpmFile.FILE),
        new RpmFile("/var/log/hello.log", RpmFile.GHOST)), rpmPackage.files());

    assertEquals(List.of(
        new RpmDependency("greeting", "GE", "0", "2.0", null, false),
        new RpmDependency("hello", "EQ", "1", "2.12.1", "3.el9", false),
        new RpmDependency("hello(x86-64)", "EQ", "1", "2.12.1", "3.el9", false)), rpmPackage.provides());
    assertEquals(List.of(
        new RpmDependency("glibc", "GE", "0", "2.34", null, false),
        new RpmDependency("/bin/sh", null, null, null, null, true)), rpmPackage.requires());
    assertEquals(List.of(new RpmDependency("hello-legacy", "LT", "0", "2.0", null, false)), rpmPackage.conflicts());
    assertEquals(List.of(new RpmDependency("hi", "LE", "1", "1.0", "1", false)), rpmPackage.obsoletes());
    assertEquals(List.of(new RpmDependency("hello-doc", null, null, null, null, false)), rpmPackage.recommends());

    assertEquals(List.of(
        new RpmChangelog("Jane Doe <jane@example.com> - 1:2.12.1-2", 1712200000L, "- Fix <build> & tests"),
        new RpmChangelog("Jane Doe <jane@example.com> - 1:2.12.1-3", 1712300000L, "- Rebuild")),
        rpmPackage.changelogs());
  }

  @Test
  public void keepsOnlyTheNewestLibcRequirement() throws Exception {
    byte[] rpm = new TestRpm("which", "2.23", "4")
        .requires("libc.so.6()(64bit)", 1 << 14, "")
        .requires("libc.so.6(GLIBC_2.4)(64bit)", 1 << 14, "")
        .requires("libc.so.6(GLIBC_2.34)(64bit)", 1 << 14, "")
        .requires("libc.so.6(GLIBC_ABI_DT_RELR)(64bit)", 1 << 14, "")
        .requires("coreutils", 0, "")
        .build();

    RpmPackage rpmPackage = RpmParser.parse(new ByteArrayInputStream(rpm));

    assertEquals(List.of(
        new RpmDependency("coreutils", null, null, null, null, false),
        new RpmDependency("libc.so.6(GLIBC_2.34)(64bit)", null, null, null, null, false)), rpmPackage.requires());
  }

  @Test
  public void computesTheHeaderRange() throws Exception {
    byte[] rpm = new TestRpm("tiny", "1.0", "1").file("/usr/bin/tiny", 3).build();

    RpmPackage rpmPackage = RpmParser.parse(new ByteArrayInputStream(rpm));

    int signatureEntries = readInt(rpm, 96 + 8);
    int signatureData = readInt(rpm, 96 + 12);
    int signatureSize = 16 + signatureEntries * 16 + signatureData;
    long start = 96 + signatureSize + (8 - signatureSize % 8) % 8;
    assertEquals(start, rpmPackage.headerStart());
    int headerEntries = readInt(rpm, (int) start + 8);
    int headerData = readInt(rpm, (int) start + 12);
    assertEquals(start + 16 + headerEntries * 16L + headerData, rpmPackage.headerEnd());
    byte[] payloadMagic = Arrays.copyOfRange(rpm, (int) rpmPackage.headerEnd(), (int) rpmPackage.headerEnd() + 2);
    assertEquals((byte) 0x1f, payloadMagic[0]);
    assertEquals((byte) 0x8b, payloadMagic[1]);
  }

  @Test
  public void namesSourcePackagesSrc() throws Exception {
    byte[] rpm = new TestRpm("hello", "2.12.1", "3").source().build();

    RpmPackage rpmPackage = RpmParser.parse(new ByteArrayInputStream(rpm));

    assertEquals("src", rpmPackage.arch());
    assertTrue(rpmPackage.isSource());
    assertEquals("hello-2.12.1-3.src.rpm", rpmPackage.fileName());
    assertEquals("0", rpmPackage.epoch());
    assertEquals("2.12.1-3", rpmPackage.componentVersion());
  }

  @Test
  public void keepsTheTenNewestChangelogEntries() throws Exception {
    TestRpm testRpm = new TestRpm("busy", "1.0", "20");
    for (int i = 20; i > 0; i--) {
      testRpm.changelog(1700000000L + i, "Packager - 1.0-" + i, "- Release " + i);
    }

    RpmPackage rpmPackage = RpmParser.parse(new ByteArrayInputStream(testRpm.build()));

    assertEquals(10, rpmPackage.changelogs().size());
    assertEquals("- Release 11", rpmPackage.changelogs().get(0).text());
    assertEquals("- Release 20", rpmPackage.changelogs().get(9).text());
  }

  @Test
  public void rejectsFilesThatAreNotPackages() {
    assertThrows(IllegalArgumentException.class,
        () -> RpmParser.parse(new ByteArrayInputStream(new byte[200])));
    assertThrows(IllegalArgumentException.class,
        () -> RpmParser.parse(new ByteArrayInputStream("not an rpm".getBytes())));
  }

  @Test
  public void rejectsTruncatedPackages() throws Exception {
    byte[] rpm = new TestRpm("tiny", "1.0", "1").build();
    byte[] truncated = Arrays.copyOf(rpm, 200);

    assertThrows(IllegalArgumentException.class, () -> RpmParser.parse(new ByteArrayInputStream(truncated)));
  }

  @Test
  public void rejectsHeadersWithEntriesOutsideTheirData() throws Exception {
    byte[] rpm = new TestRpm("tiny", "1.0", "1").build();
    int signatureEntries = readInt(rpm, 96 + 8);
    int signatureData = readInt(rpm, 96 + 12);
    int signatureSize = 16 + signatureEntries * 16 + signatureData;
    int start = 96 + signatureSize + (8 - signatureSize % 8) % 8;
    rpm[start + 16 + 8] = 0x7f;

    assertThrows(IllegalArgumentException.class, () -> RpmParser.parse(new ByteArrayInputStream(rpm)));
  }

  @Test
  public void parsesFileNames() {
    assertEquals(new RpmFileName("hello", "2.12.1", "3.el9", "x86_64"),
        RpmFileName.parse("hello-2.12.1-3.el9.x86_64.rpm").orElseThrow());
    assertEquals(new RpmFileName("python3-foo-bar", "1.0~rc1", "0.1.fc40", "noarch"),
        RpmFileName.parse("python3-foo-bar-1.0~rc1-0.1.fc40.noarch.rpm").orElseThrow());
    assertFalse(RpmFileName.parse("hello.rpm").isPresent());
  }

  @Test
  public void splitsVersionsOfDependencies() {
    assertEquals(new RpmDependency.Evr("2", "1.0", "3"), RpmDependency.Evr.parse("2:1.0-3"));
    assertEquals(new RpmDependency.Evr("0", "1.0", null), RpmDependency.Evr.parse("1.0"));
    assertEquals(new RpmDependency.Evr("0", "1.0-rc", "1"), RpmDependency.Evr.parse("1.0-rc-1"));
  }

  private static int readInt(final byte[] bytes, final int position) {
    return ((bytes[position] & 0xff) << 24) | ((bytes[position + 1] & 0xff) << 16)
        | ((bytes[position + 2] & 0xff) << 8) | (bytes[position + 3] & 0xff);
  }
}
