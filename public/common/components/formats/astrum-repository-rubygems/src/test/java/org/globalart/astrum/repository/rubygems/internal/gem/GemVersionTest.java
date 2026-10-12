package org.globalart.astrum.repository.rubygems.internal.gem;

import java.util.ArrayList;
import java.util.List;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;

class GemVersionTest
{
  @Test
  void comparesLikeRubygems() {
    assertThat(GemVersion.compare("1.0", "1.0.0"), is(0));
    assertThat(GemVersion.compare("1.10", "1.9"), is(1));
    assertThat(GemVersion.compare("1.0.a", "1.0"), is(-1));
    assertThat(GemVersion.compare("1.0.0.pre", "1.0.0.rc1"), is(-1));
    assertThat(GemVersion.compare("1.0.0.rc1", "1.0.0.rc10"), is(-1));
    assertThat(GemVersion.compare("1.0.0-pre", "1.0.0.pre.pre"), is(0));
    assertThat(GemVersion.compare("2", "10"), is(-1));
    assertThat(GemVersion.compare("1.0.a10", "1.0.b1"), is(-1));
  }

  @Test
  void sortsVersions() {
    List<String> versions = new ArrayList<>(List.of("1.10.0", "1.2.0", "1.2.0.beta", "0.9", "1.2.0.rc.1"));
    versions.sort(GemVersion.COMPARATOR);
    assertThat(versions, contains("0.9", "1.2.0.beta", "1.2.0.rc.1", "1.2.0", "1.10.0"));
  }

  @Test
  void detectsPrereleases() {
    assertThat(GemVersion.isPrerelease("1.0.0"), is(false));
    assertThat(GemVersion.isPrerelease("1.0.0.pre"), is(true));
    assertThat(GemVersion.isPrerelease("2.0.0.rc1"), is(true));
  }

  @Test
  void validatesVersions() {
    assertThat(GemVersion.isValid("1.0.0"), is(true));
    assertThat(GemVersion.isValid("1.0.0-beta.2"), is(true));
    assertThat(GemVersion.isValid("1.0.0.pre"), is(true));
    assertThat(GemVersion.isValid("v1.0"), is(false));
    assertThat(GemVersion.isValid("1.0/../x"), is(false));
    assertThat(GemVersion.isValid(""), is(false));
  }
}
