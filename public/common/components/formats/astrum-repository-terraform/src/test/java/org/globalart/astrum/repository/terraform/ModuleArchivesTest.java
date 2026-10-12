package org.globalart.astrum.repository.terraform;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

public class ModuleArchivesTest
{
  @Test
  public void readsTheArchiveTypeFromTheName() {
    assertThat(ModuleArchives.extension("vpc.tar.gz")).contains("tar.gz");
    assertThat(ModuleArchives.extension("vpc.TGZ")).contains("tar.gz");
    assertThat(ModuleArchives.extension("vpc.zip")).contains("zip");
    assertThat(ModuleArchives.extension("vpc.txz")).contains("tar.xz");
    assertThat(ModuleArchives.extension(".zip")).isEmpty();
    assertThat(ModuleArchives.extension("vpc.tar")).isEmpty();
    assertThat(ModuleArchives.fromArchiveParameter("tgz")).contains("tar.gz");
    assertThat(ModuleArchives.fromArchiveParameter("rar")).isEmpty();
  }

  @Test
  public void detectsTheArchiveTypeFromTheContent() {
    assertThat(ModuleArchives.sniff(new byte[]{'P', 'K', 3, 4, 0})).contains("zip");
    assertThat(ModuleArchives.sniff(new byte[]{(byte) 0x1f, (byte) 0x8b, 8})).contains("tar.gz");
    assertThat(ModuleArchives.sniff(new byte[]{'B', 'Z', 'h', '9'})).contains("tar.bz2");
    assertThat(ModuleArchives.sniff("plain".getBytes())).isEmpty();
  }
}
