package org.globalart.astrum.repository.r.internal.util;

import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.util.List;

import org.globalart.astrum.repository.r.internal.util.RPackagesUtils.MergeResult;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;

public class RPackagesUtilsTest
{
  private static final String HOSTED = """
      Package: praise
      Version: 1.0.0
      MD5sum: 1111

      Package: internal
      Version: 0.1.0
      """;

  private static final String CRAN = """
      Package: glue
      Version: 1.8.0
      Imports: methods
      License: MIT + file LICENSE

      Package: praise
      Version: 1.0.0
      MD5sum: 9318724cec0454884b5f762bee2da6a1

      Package: class
      Version: 7.3-23

      Package: class
      Version: 7.3-24
      Path: 4.7.0/Recommended
      """;

  @Test
  public void mergesMembersInOrderAndKeepsTheFirstMemberEntries() {
    MergeResult result = RPackagesUtils.merge(List.of(HOSTED, CRAN));

    assertThat(result.packageCount(), is(5));
    assertThat(result.packages(), is(equalTo("""
        Package: praise
        Version: 1.0.0
        MD5sum: 1111

        Package: internal
        Version: 0.1.0

        Package: glue
        Version: 1.8.0
        Imports: methods
        License: MIT + file LICENSE

        Package: class
        Version: 7.3-23

        Package: class
        Version: 7.3-24
        Path: 4.7.0/Recommended
        """)));
  }

  @Test
  public void readsPlainAndGzippedPackages() throws IOException {
    byte[] gzipped = RPackagesUtils.toBytes(CRAN, true);

    assertThat(RPackagesUtils.readPackages(new ByteArrayInputStream(gzipped)), is(equalTo(CRAN)));
    assertThat(RPackagesUtils.readPackages(new ByteArrayInputStream(RPackagesUtils.toBytes(CRAN, false))),
        is(equalTo(CRAN)));
  }

  @Test
  public void findsPackageNames() {
    assertThat(RPackagesUtils.stanzas(CRAN).stream().map(RPackagesUtils::packageName).toList(),
        is(equalTo(List.of("glue", "praise", "class", "class"))));
  }
}
