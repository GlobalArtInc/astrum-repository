
package org.globalart.astrum.repository.r.internal.util;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.Map;
import java.util.zip.GZIPOutputStream;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.repository.InvalidContentException;

import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.ISO_8859_1;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.repository.r.internal.util.RDescriptionUtils.extractDescriptionFromArchive;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;

public class RDescriptionUtilsTest
{
  private static final String DESCRIPTION = """
      Package: RPostgreSQL
      Version: 0.4-1
      Title: R interface to the PostgreSQL database system
      Author: Joe Conway, Dirk Eddelbuettel, Tomoaki Nishiyama, Sameer Kumar
              Prayaga (during 2008), Neil Tiffin
      Depends: R (>= 2.9.0), methods, DBI (>= 0.1-4)
      License: GPL-2 | file LICENSE
      NeedsCompilation: yes
      """;

  @Test
  public void extractsDescriptionFromTarGz() throws IOException {
    byte[] archive = tarGz("RPostgreSQL/R/PostgreSQL.R", "x <- 1\n",
        "RPostgreSQL/inst/DESCRIPTION", "Package: wrong\n",
        "RPostgreSQL/DESCRIPTION", DESCRIPTION);

    assertAttributes(extractDescriptionFromArchive("RPostgreSQL_0.4-1.tar.gz", supplier(archive)));
    assertAttributes(extractDescriptionFromArchive("RPostgreSQL_0.4-1.tgz", supplier(archive)));
  }

  @Test
  public void extractsDescriptionFromZip() throws IOException {
    byte[] archive = zip("RPostgreSQL/libs/RPostgreSQL.dll", "binary", "RPostgreSQL/DESCRIPTION", DESCRIPTION);

    assertAttributes(extractDescriptionFromArchive("RPostgreSQL_0.4-1.zip", supplier(archive)));
  }

  @Test
  public void decodesLatin1Descriptions() throws IOException {
    byte[] description = "Package: café\nVersion: 1.0\nEncoding: latin1\n".getBytes(ISO_8859_1);
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (TarArchiveOutputStream tar = new TarArchiveOutputStream(new GZIPOutputStream(out))) {
      TarArchiveEntry entry = new TarArchiveEntry("cafe/DESCRIPTION");
      entry.setSize(description.length);
      tar.putArchiveEntry(entry);
      tar.write(description);
      tar.closeArchiveEntry();
    }

    Map<String, String> attributes = extractDescriptionFromArchive("cafe_1.0.tar.gz", supplier(out.toByteArray()));

    assertThat(attributes.get("Package"), is(equalTo("café")));
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsArchivesWithoutDescription() throws IOException {
    extractDescriptionFromArchive("pkg_1.0.zip", supplier(zip("pkg/NAMESPACE", "export(f)\n")));
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsCorruptArchives() {
    extractDescriptionFromArchive("pkg_1.0.tar.gz", supplier("not a tarball".getBytes(UTF_8)));
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsUnsupportedExtensions() {
    extractDescriptionFromArchive("pkg_1.0.rar", supplier(new byte[0]));
  }

  private static void assertAttributes(final Map<String, String> attributes) {
    assertThat(attributes.get("Package"), is(equalTo("RPostgreSQL")));
    assertThat(attributes.get("Version"), is(equalTo("0.4-1")));
    assertThat(attributes.get("Author"), is(equalTo("Joe Conway, Dirk Eddelbuettel, Tomoaki Nishiyama, Sameer Kumar\n"
        + "        Prayaga (during 2008), Neil Tiffin")));
    assertThat(attributes.get("Depends"), is(equalTo("R (>= 2.9.0), methods, DBI (>= 0.1-4)")));
    assertThat(attributes.get("License"), is(equalTo("GPL-2 | file LICENSE")));
    assertThat(attributes.get("NeedsCompilation"), is(equalTo("yes")));
  }

  private static InputStreamSupplier supplier(final byte[] bytes) {
    return () -> new ByteArrayInputStream(bytes);
  }

  private static byte[] tarGz(final String... entries) throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (TarArchiveOutputStream tar = new TarArchiveOutputStream(new GZIPOutputStream(out))) {
      for (int i = 0; i < entries.length; i += 2) {
        byte[] content = entries[i + 1].getBytes(UTF_8);
        TarArchiveEntry entry = new TarArchiveEntry(entries[i]);
        entry.setSize(content.length);
        tar.putArchiveEntry(entry);
        tar.write(content);
        tar.closeArchiveEntry();
      }
    }
    return out.toByteArray();
  }

  private static byte[] zip(final String... entries) throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (ZipOutputStream zip = new ZipOutputStream(out)) {
      for (int i = 0; i < entries.length; i += 2) {
        zip.putNextEntry(new ZipEntry(entries[i]));
        zip.write(entries[i + 1].getBytes(UTF_8));
        zip.closeEntry();
      }
    }
    return out.toByteArray();
  }
}
