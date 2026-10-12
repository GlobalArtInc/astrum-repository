package org.globalart.astrum.repository.yum.internal.rpm;

import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class RpmVersionsTest
{
  @Test
  public void comparesLikeRpm() {
    assertEquals(0, RpmVersions.compare("1.0", "1.0"));
    assertEquals(-1, RpmVersions.compare("1.0", "1.0.1"));
    assertEquals(1, RpmVersions.compare("2.34", "2.4"));
    assertEquals(1, RpmVersions.compare("1.010", "1.9"));
    assertEquals(0, RpmVersions.compare("1.01", "1.1"));
    assertEquals(-1, RpmVersions.compare("1.0~rc1", "1.0"));
    assertEquals(-1, RpmVersions.compare("1.0~rc1", "1.0~rc2"));
    assertEquals(1, RpmVersions.compare("1.0^git1", "1.0"));
    assertEquals(-1, RpmVersions.compare("1.0^git1", "1.0.1"));
    assertEquals(1, RpmVersions.compare("1.0a", "1.0"));
    assertEquals(-1, RpmVersions.compare("a", "1"));
    assertEquals(1, RpmVersions.compare("2.34", "ABI_DT_RELR"));
  }

  @Test
  public void ordersLibcRequiresByTheirGlibcVersion() {
    assertEquals(1, RpmVersions.compareLibcRequires("libc.so.6(GLIBC_2.34)(64bit)", "libc.so.6(GLIBC_2.4)(64bit)"));
    assertEquals(1, RpmVersions.compareLibcRequires("libc.so.6(GLIBC_2.0)", "libc.so.6"));
    assertEquals(-1, RpmVersions.compareLibcRequires("libc.so.6(GLIBC_ABI_DT_RELR)", "libc.so.6(GLIBC_2.4)"));
  }
}
