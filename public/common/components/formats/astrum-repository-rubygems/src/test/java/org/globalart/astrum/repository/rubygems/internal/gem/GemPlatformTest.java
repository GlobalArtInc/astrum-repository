package org.globalart.astrum.repository.rubygems.internal.gem;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class GemPlatformTest
{
  @Test
  void parsesPlatformsLikeRubygems() {
    assertThat(GemPlatform.parse("x86_64-linux"), is(new GemPlatform("x86_64", "linux", null)));
    assertThat(GemPlatform.parse("x86_64-linux-musl"), is(new GemPlatform("x86_64", "linux", "musl")));
    assertThat(GemPlatform.parse("arm64-darwin-22"), is(new GemPlatform("arm64", "darwin", "22")));
    assertThat(GemPlatform.parse("x64-mingw-ucrt"), is(new GemPlatform("x64", "mingw", "ucrt")));
    assertThat(GemPlatform.parse("i686-linux"), is(new GemPlatform("x86", "linux", null)));
    assertThat(GemPlatform.parse("java"), is(new GemPlatform(null, "java", null)));
    assertThat(GemPlatform.parse("x86-mswin32-60"), is(new GemPlatform("x86", "mswin32", "60")));
    assertThat(GemPlatform.parse("universal-darwin-9"), is(new GemPlatform("universal", "darwin", "9")));
  }

  @Test
  void printsPlatforms() {
    assertThat(GemPlatform.parse("x86_64-linux-gnu").toString(), is("x86_64-linux-gnu"));
    assertThat(GemPlatform.parse("java").toString(), is("java"));
    assertThat(GemPlatform.parse("java").cpu(), is(nullValue()));
  }

  @Test
  void normalizesRuby() {
    assertThat(GemPlatform.normalize(null), is("ruby"));
    assertThat(GemPlatform.normalize(" "), is("ruby"));
    assertThat(GemPlatform.isRuby("ruby"), is(true));
    assertThat(GemPlatform.isRuby("java"), is(false));
  }
}
