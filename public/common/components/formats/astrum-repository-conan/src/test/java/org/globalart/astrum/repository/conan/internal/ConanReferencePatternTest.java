package org.globalart.astrum.repository.conan.internal;

import org.globalart.astrum.repository.conan.ConanCoords;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;

public class ConanReferencePatternTest
{
  private final ConanCoords zlib = new ConanCoords("zlib", "1.3.1", null, null, null, null, null, null);

  private final ConanCoords hello = new ConanCoords("hello", "0.1", "acme", "stable", null, null, null, null);

  @Test
  public void exactName() {
    assertThat(ConanReferencePattern.of("zlib", true).matches(zlib), is(true));
    assertThat(ConanReferencePattern.of("zli", true).matches(zlib), is(false));
  }

  @Test
  public void wildcards() {
    assertThat(ConanReferencePattern.of("zl*", true).matches(zlib), is(true));
    assertThat(ConanReferencePattern.of("zlib/*", true).matches(zlib), is(true));
    assertThat(ConanReferencePattern.of("zlib/1.2*", true).matches(zlib), is(false));
    assertThat(ConanReferencePattern.of("*", true).matches(hello), is(true));
    assertThat(ConanReferencePattern.of(null, true).matches(zlib), is(true));
  }

  @Test
  public void userAndChannel() {
    assertThat(ConanReferencePattern.of("hello/0.1@acme/stable", true).matches(hello), is(true));
    assertThat(ConanReferencePattern.of("hello/0.1@acme/*", true).matches(hello), is(true));
    assertThat(ConanReferencePattern.of("hello/0.1@other/*", true).matches(hello), is(false));
  }

  @Test
  public void caseSensitivity() {
    assertThat(ConanReferencePattern.of("ZLIB", true).matches(zlib), is(true));
    assertThat(ConanReferencePattern.of("ZLIB", false).matches(zlib), is(false));
  }

  @Test
  public void namePrefilter() {
    assertThat(ConanReferencePattern.of("zlib/1.*", true).nameLikePattern(), is(equalTo("zlib")));
    assertThat(ConanReferencePattern.of("Open_SSL*", true).nameLikePattern(), is(equalTo("open\\_ssl%")));
    assertThat(ConanReferencePattern.of(null, true).nameLikePattern(), is(equalTo("%")));
  }
}
