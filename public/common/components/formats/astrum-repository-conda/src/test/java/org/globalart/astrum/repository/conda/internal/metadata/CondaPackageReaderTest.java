package org.globalart.astrum.repository.conda.internal.metadata;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.OutputStream;
import java.util.List;
import java.util.Map;
import java.util.zip.CRC32;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import org.globalart.astrum.repository.conda.AssetKind;

import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream;
import org.apache.commons.compress.compressors.bzip2.BZip2CompressorOutputStream;
import org.apache.commons.compress.compressors.zstandard.ZstdCompressorOutputStream;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.fail;

public class CondaPackageReaderTest
{
  private static final String INDEX = "{\"name\":\"demo\",\"version\":\"1.0\",\"build\":\"h0_0\",\"build_number\":0,"
      + "\"depends\":[\"python >=3.9\"],\"subdir\":\"noarch\",\"timestamp\":1700000000000}";

  @Test
  public void readsIndexFromTarBz2() throws Exception {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (BZip2CompressorOutputStream bzip = new BZip2CompressorOutputStream(out)) {
      writeTar(bzip, Map.of("info/about.json", "{}", "info/index.json", INDEX, "site-packages/demo.py", "x = 1"));
    }
    Map<String, Object> index = CondaPackageReader.readIndex(
        new ByteArrayInputStream(out.toByteArray()), AssetKind.ARCH_TAR_PACKAGE);
    assertEquals("demo", index.get("name"));
    assertEquals(List.of("python >=3.9"), index.get("depends"));
    assertEquals(1700000000000L, ((Number) index.get("timestamp")).longValue());
  }

  @Test
  public void readsIndexFromCondaArchive() throws Exception {
    ByteArrayOutputStream info = new ByteArrayOutputStream();
    try (ZstdCompressorOutputStream zstd = new ZstdCompressorOutputStream(info)) {
      writeTar(zstd, Map.of("info/index.json", INDEX));
    }
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (ZipOutputStream zip = new ZipOutputStream(out)) {
      stored(zip, "metadata.json", "{\"conda_pkg_format_version\": 2}".getBytes(UTF_8));
      stored(zip, "pkg-demo-1.0-h0_0.tar.zst", new byte[0]);
      stored(zip, "info-demo-1.0-h0_0.tar.zst", info.toByteArray());
    }
    Map<String, Object> index = CondaPackageReader.readIndex(
        new ByteArrayInputStream(out.toByteArray()), AssetKind.ARCH_CONDA_PACKAGE);
    assertEquals("h0_0", index.get("build"));
    assertEquals("noarch", index.get("subdir"));
  }

  @Test
  public void rejectsArchivesWithoutIndex() throws Exception {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (BZip2CompressorOutputStream bzip = new BZip2CompressorOutputStream(out)) {
      writeTar(bzip, Map.of("info/about.json", "{}"));
    }
    try {
      CondaPackageReader.readIndex(new ByteArrayInputStream(out.toByteArray()), AssetKind.ARCH_TAR_PACKAGE);
      fail();
    }
    catch (InvalidCondaPackageException expected) {
      assertEquals("Package does not contain info/index.json", expected.getMessage());
    }
  }

  @Test(expected = InvalidCondaPackageException.class)
  public void rejectsGarbage() throws Exception {
    CondaPackageReader.readIndex(new ByteArrayInputStream("<html>".getBytes(UTF_8)), AssetKind.ARCH_TAR_PACKAGE);
  }

  private static void writeTar(final OutputStream out, final Map<String, String> files) throws IOException {
    TarArchiveOutputStream tar = new TarArchiveOutputStream(out);
    for (Map.Entry<String, String> file : files.entrySet()) {
      byte[] data = file.getValue().getBytes(UTF_8);
      TarArchiveEntry entry = new TarArchiveEntry(file.getKey());
      entry.setSize(data.length);
      tar.putArchiveEntry(entry);
      tar.write(data);
      tar.closeArchiveEntry();
    }
    tar.finish();
  }

  private static void stored(final ZipOutputStream zip, final String name, final byte[] data) throws IOException {
    ZipEntry entry = new ZipEntry(name);
    entry.setMethod(ZipEntry.STORED);
    entry.setSize(data.length);
    CRC32 crc = new CRC32();
    crc.update(data);
    entry.setCrc(crc.getValue());
    zip.putNextEntry(entry);
    zip.write(data);
    zip.closeEntry();
  }
}
