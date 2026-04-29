package org.globalart.astrum.repository.r.internal.util;

import java.io.IOException;
import java.io.InputStream;
import java.util.List;
import java.util.Map;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;

public class RPackagesBuilderTest
{
  @Test
  public void parsesDescriptionWithContinuationLines() throws IOException {
    Map<String, String> description = RMetadataUtils.parseDescriptionFile(resource("DESCRIPTION"));

    assertThat(description.get("Package"), is(equalTo("ggplot2")));
    assertThat(description.get("Version"), is(equalTo("2.1.0")));
    assertThat(description.get("Imports"), is(equalTo("digest, grid, gtable (>= 0.1.1), MASS, plyr (>= 1.7.1),\n"
        + "        reshape2, scales (>= 0.3.0), stats")));
    assertThat(description.get("Built"), is(equalTo("R 3.3.0; ; 2016-05-05 12:00:56 UTC; unix")));
  }

  @Test
  public void writesStandardFieldsOfTheGreatestVersion() throws IOException {
    RPackagesBuilder builder = new RPackagesBuilder();
    builder.append(Map.of("Package", "praise", "Version", "1.0.0", "Suggests", "testthat",
        "License", "MIT + file LICENSE", "NeedsCompilation", "no", "Title", "Praise Users"), "9318724c");
    builder.append(Map.of("Package", "praise", "Version", "0.9", "License", "MIT"), "00000000");
    builder.append(RMetadataUtils.parseDescriptionFile(resource("DESCRIPTION")), "abcdef");
    builder.append(Map.of("Version", "1.0"), null);

    assertThat(builder.buildPackages(), is(equalTo(String.join("\n",
        "Package: ggplot2",
        "Version: 2.1.0",
        "Depends: R (>= 3.1)",
        "Imports: digest, grid, gtable (>= 0.1.1), MASS, plyr (>= 1.7.1),",
        "        reshape2, scales (>= 0.3.0), stats",
        "Suggests: covr, ggplot2movies, hexbin, Hmisc, lattice, mapproj, maps,",
        "        maptools, mgcv, multcomp, nlme, testthat (>= 0.11.0), quantreg,",
        "        knitr, rpart, rmarkdown, svglite",
        "Enhances: sp",
        "License: GPL-2",
        "MD5sum: abcdef",
        "Built: R 3.3.0; ; 2016-05-05 12:00:56 UTC; unix",
        "NeedsCompilation: no",
        "",
        "Package: praise",
        "Version: 1.0.0",
        "Suggests: testthat",
        "License: MIT + file LICENSE",
        "MD5sum: 9318724c",
        "NeedsCompilation: no",
        ""))));
  }

  @Test
  public void keepsTheNewestVersionRegardlessOfOrder() {
    RPackagesBuilder builder = new RPackagesBuilder();
    builder.append(Map.of("Package", "glue", "Version", "1.10.0"), null);
    builder.append(Map.of("Package", "glue", "Version", "1.9.0"), null);
    builder.append(Map.of("Package", "glue", "Version", "1.6.2"), null);

    assertThat(builder.getPackageInformation().get("glue").get("Version"), is(equalTo("1.10.0")));
  }

  @Test
  public void readsWhatItWrites() throws IOException {
    List<Map<String, String>> records = RMetadataUtils.parseDcf(new String(resource("PACKAGES")));
    RPackagesBuilder builder = new RPackagesBuilder();
    records.forEach(record -> builder.append(record, null));

    List<Map<String, String>> reparsed = RMetadataUtils.parseDcf(builder.buildPackages());

    assertThat(reparsed.stream().map(record -> record.get("Package")).toList(),
        contains("A3", "ABCanalysis", "abbyyR", "abc"));
    assertThat(reparsed.get(0).get("Depends"), is(equalTo("R (>= 2.15.0), xtable, pbapply")));
  }

  private static byte[] resource(final String name) throws IOException {
    try (InputStream in = RPackagesBuilderTest.class.getResourceAsStream(name)) {
      return in.readAllBytes();
    }
  }
}
