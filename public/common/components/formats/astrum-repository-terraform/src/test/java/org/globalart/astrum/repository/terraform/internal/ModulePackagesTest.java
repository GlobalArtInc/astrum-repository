package org.globalart.astrum.repository.terraform.internal;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream;
import org.apache.commons.compress.compressors.gzip.GzipCompressorOutputStream;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.assertj.core.api.Assertions.assertThat;

public class ModulePackagesTest
{
  @Test
  public void resolvesTheTopDirectoryOfATarball() throws Exception {
    byte[] tarball = tarGz("terraform-aws-vpc-5.0.0/main.tf", "terraform-aws-vpc-5.0.0/modules/flow-log/main.tf");

    ModulePackages.Layout layout = ModulePackages.layout(new ByteArrayInputStream(tarball), "tar.gz").orElseThrow();

    assertThat(layout.rootFiles()).isFalse();
    assertThat(layout.singleTopDirectory()).contains("terraform-aws-vpc-5.0.0");
    assertThat(layout.resolve("*")).contains("terraform-aws-vpc-5.0.0");
    assertThat(layout.resolve("*/modules/flow-log")).contains("terraform-aws-vpc-5.0.0/modules/flow-log");
    assertThat(layout.resolve("modules")).contains("modules");
    assertThat(layout.resolve("*/missing")).isEmpty();
  }

  @Test
  public void readsZipArchivesWithFilesAtTheRoot() throws Exception {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (ZipOutputStream zip = new ZipOutputStream(out)) {
      for (String name : new String[]{"main.tf", "modules/a/main.tf", "modules/b/main.tf"}) {
        zip.putNextEntry(new ZipEntry(name));
        zip.write("# module".getBytes(UTF_8));
        zip.closeEntry();
      }
    }

    ModulePackages.Layout layout = ModulePackages.layout(new ByteArrayInputStream(out.toByteArray()), "zip")
        .orElseThrow();

    assertThat(layout.rootFiles()).isTrue();
    assertThat(layout.singleTopDirectory()).isEmpty();
    assertThat(layout.resolve("modules/*")).isEmpty();
    assertThat(layout.directories()).contains("modules", "modules/a", "modules/b");
  }

  @Test
  public void leavesUnreadableArchiveTypesAlone() throws Exception {
    assertThat(ModulePackages.layout(new ByteArrayInputStream(new byte[0]), "tar.xz")).isEmpty();
    assertThat(ModulePackages.isGlob("*/modules")).isTrue();
    assertThat(ModulePackages.isGlob("modules/vpc")).isFalse();
  }

  private static byte[] tarGz(final String... names) throws Exception {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (TarArchiveOutputStream tar = new TarArchiveOutputStream(new GzipCompressorOutputStream(out))) {
      for (String name : names) {
        byte[] content = "# module".getBytes(UTF_8);
        TarArchiveEntry entry = new TarArchiveEntry(name);
        entry.setSize(content.length);
        tar.putArchiveEntry(entry);
        tar.write(content);
        tar.closeArchiveEntry();
      }
    }
    return out.toByteArray();
  }
}
