package org.globalart.astrum.repository.pypi.internal;

import org.junit.Test;

import static org.junit.Assert.assertEquals;

public class PyPiPathAndFileUtilsTest
{
  @Test
  public void normalizesNamesPerPep503() {
    assertEquals("my-package", PyPiPathUtils.normalizeName("My_Package"));
    assertEquals("my-package", PyPiPathUtils.normalizeName("my.package"));
    assertEquals("my-package", PyPiPathUtils.normalizeName("My--_.Package"));
    assertEquals("charset-normalizer", PyPiPathUtils.normalizeName("Charset_Normalizer"));
  }

  @Test
  public void extractsPackageNameFromPath() {
    assertEquals("requests", PyPiPathUtils.packageNameFromPath("/packages/requests/2.32.5/requests-2.32.5.tar.gz"));
    assertEquals("requests", PyPiPathUtils.packageNameFromPath("packages/requests/2.32.5/requests-2.32.5.tar.gz"));
  }

  @Test
  public void extractsVersionsFromFilenames() {
    assertEquals("2.32.5", PyPiFileUtils.extractVersionFromFilename("requests-2.32.5-py3-none-any.whl"));
    assertEquals("3.5.2", PyPiFileUtils.extractVersionFromFilename(
        "charset_normalizer-3.5.2-cp39-cp39-macosx_10_9_universal2.whl"));
    assertEquals("1.2.0", PyPiFileUtils.extractVersionFromFilename("sample-1.2.0.tar.gz"));
    assertEquals("1.2.0", PyPiFileUtils.extractVersionFromFilename("sample-1.2.0-py2.7.egg"));
    assertEquals("sample", PyPiFileUtils.extractNameFromFilename("sample-1.2.0.tar.gz"));
  }
}
