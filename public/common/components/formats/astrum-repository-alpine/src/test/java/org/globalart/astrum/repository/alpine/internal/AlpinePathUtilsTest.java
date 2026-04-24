package org.globalart.astrum.repository.alpine.internal;

import java.util.Optional;

import org.globalart.astrum.repository.alpine.internal.AlpinePathUtils.NameAndVersion;

import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertThrows;

public class AlpinePathUtilsTest
{
  @Test
  public void splitsPackageFileNames() {
    assertEquals(Optional.of(new NameAndVersion("zlib", "1.3.2-r0")),
        AlpinePathUtils.nameAndVersion("zlib-1.3.2-r0.apk"));
    assertEquals(Optional.of(new NameAndVersion("a2ps-dev", "4.14-r5")),
        AlpinePathUtils.nameAndVersion("a2ps-dev-4.14-r5.apk"));
    assertEquals(Optional.of(new NameAndVersion("lua5.3", "5.3.6-r6")),
        AlpinePathUtils.nameAndVersion("lua5.3-5.3.6-r6.apk"));
    assertEquals(Optional.of(new NameAndVersion("fcgi++", "2.4.2-r4")),
        AlpinePathUtils.nameAndVersion("fcgi++-2.4.2-r4.apk"));
    assertEquals(Optional.of(new NameAndVersion("py3-2to3", "3.12.1_rc1-r0")),
        AlpinePathUtils.nameAndVersion("py3-2to3-3.12.1_rc1-r0.apk"));
    assertEquals(Optional.empty(), AlpinePathUtils.nameAndVersion("APKINDEX.tar.gz"));
  }

  @Test
  public void normalizesPaths() {
    assertEquals("/v3.20/main/x86_64", AlpinePathUtils.normalize("v3.20//main/x86_64/"));
    assertEquals("/v3.20/main/x86_64", AlpinePathUtils.directory("/v3.20/main/x86_64/zlib-1.3.2-r0.apk"));
    assertEquals("x86_64", AlpinePathUtils.lastSegment("/v3.20/main/x86_64"));
    assertEquals("/v3.20/main/x86_64/APKINDEX.tar.gz", AlpinePathUtils.indexPath("/v3.20/main/x86_64"));
    assertThrows(IllegalArgumentException.class, () -> AlpinePathUtils.normalize("v3.20/../x86_64"));
  }
}
