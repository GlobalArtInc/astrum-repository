package org.globalart.astrum.repository.r.internal.util;

import java.util.Map;

import org.globalart.astrum.repository.InvalidContentException;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.arrayContaining;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;

public class RPathUtilsTest
{
  @Test
  public void splitsPaths() {
    assertThat(RPathUtils.directory("src/contrib/praise_1.0.0.tar.gz"), is(equalTo("/src/contrib")));
    assertThat(RPathUtils.namespace("/src/contrib/praise_1.0.0.tar.gz"), is(equalTo("src/contrib")));
    assertThat(RPathUtils.fileName("/src/contrib/praise_1.0.0.tar.gz"), is(equalTo("praise_1.0.0.tar.gz")));
    assertThat(RPathUtils.buildPath("/src/contrib/", "praise_1.0.0.tar.gz"),
        is(equalTo("/src/contrib/praise_1.0.0.tar.gz")));
    assertThat(RPathUtils.packagesPath("/bin/windows/contrib/4.4", true),
        is(equalTo("/bin/windows/contrib/4.4/PACKAGES.gz")));
  }

  @Test
  public void recognisesArchives() {
    assertThat(RPathUtils.isArchive("/src/contrib/praise_1.0.0.tar.gz"), is(true));
    assertThat(RPathUtils.isArchive("/bin/macosx/contrib/4.4/praise_1.0.0.tgz"), is(true));
    assertThat(RPathUtils.isArchive("/bin/windows/contrib/4.4/praise_1.0.0.ZIP"), is(true));
    assertThat(RPathUtils.isArchive("/src/contrib/PACKAGES.gz"), is(false));
    assertThat(RPathUtils.isArchive("/src/contrib/.tar.gz"), is(false));
    assertThat(RPathUtils.nameAndVersion("data.table_1.17.8.tar.gz").orElseThrow(),
        arrayContaining("data.table", "1.17.8"));
    assertThat(RPathUtils.nameAndVersion("PACKAGES.gz").isPresent(), is(false));
  }

  @Test
  public void validatesUploadPaths() {
    PackageValidator.validateArchiveUploadPath("/src/contrib/praise_1.0.0.tar.gz");
    PackageValidator.validateArchiveFileName("/src/contrib/praise_1.0.0.tar.gz",
        Map.of("Package", "praise", "Version", "1.0.0"));
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsShallowPaths() {
    PackageValidator.validateArchiveUploadPath("/praise_1.0.0.tar.gz");
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsOtherExtensions() {
    PackageValidator.validateArchiveUploadPath("/src/contrib/praise_1.0.0.tar.bz2");
  }

  @Test(expected = InvalidContentException.class)
  public void rejectsFileNamesThatDoNotMatchTheDescription() {
    PackageValidator.validateArchiveFileName("/src/contrib/praise.tar.gz",
        Map.of("Package", "praise", "Version", "1.0.0"));
  }
}
