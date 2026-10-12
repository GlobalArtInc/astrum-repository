package org.globalart.astrum.repository.swift;

import java.util.ArrayList;
import java.util.Collections;
import java.util.List;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;

class SwiftVersionTest
{
  @Test
  void acceptsSemanticVersionsOnly() {
    for (String valid : List.of("0.0.0", "1.2.3", "10.20.30", "1.0.0-alpha", "1.0.0-alpha.1", "1.0.0-0.3.7",
        "1.0.0-x.7.z.92", "1.0.0+20130313144700", "1.0.0-beta+exp.sha.5114f85", "1.0.0-rc-1")) {
      assertThat(valid, SwiftVersion.isValid(valid), is(true));
    }
    for (String invalid : List.of("", "1", "1.0", "v1.0.0", "01.0.0", "1.0.0-", "1.0.0-01", "1.0.0+", "1.0.0.zip",
        "1.0.0 ", "1.0.0-beta..1", "99999999999999999999.0.0")) {
      assertThat(invalid, SwiftVersion.isValid(invalid), is(false));
    }
    assertThat(SwiftVersion.isValid(null), is(false));
  }

  @Test
  void ordersByPrecedence() {
    List<String> versions = new ArrayList<>(List.of("1.0.0", "1.0.0-rc.1", "1.0.0-beta.11", "1.0.0-alpha.beta",
        "1.0.0-beta", "1.0.0-alpha.1", "1.0.0-beta.2", "1.0.0-alpha", "2.1.1", "2.1.0", "2.0.0", "10.0.0"));
    Collections.shuffle(versions);
    versions.sort(SwiftVersion.PRECEDENCE);
    assertThat(versions, contains("1.0.0-alpha", "1.0.0-alpha.1", "1.0.0-alpha.beta", "1.0.0-beta", "1.0.0-beta.2",
        "1.0.0-beta.11", "1.0.0-rc.1", "1.0.0", "2.0.0", "2.1.0", "2.1.1", "10.0.0"));
  }

  @Test
  void ignoresBuildMetadataForPrecedence() {
    assertThat(SwiftVersion.parse("1.0.0+1").orElseThrow().compareTo(SwiftVersion.parse("1.0.0+2").orElseThrow()),
        is(0));
    assertThat(SwiftVersion.parse("1.0.0-rc.1").orElseThrow().isPrerelease(), is(true));
    assertThat(SwiftVersion.parse("1.0.0+build").orElseThrow().isPrerelease(), is(false));
  }
}
