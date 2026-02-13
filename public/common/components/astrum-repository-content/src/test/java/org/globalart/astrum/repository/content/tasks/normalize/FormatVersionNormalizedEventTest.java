
package org.globalart.astrum.repository.content.tasks.normalize;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

public class FormatVersionNormalizedEventTest
{
  @Test
  public void testConstructorWithFormat() {
    FormatVersionNormalizedEvent underTest = new FormatVersionNormalizedEvent("maven2");
    assertThat(underTest.getFormat(), is("maven2"));
  }

  @Test
  public void testDefaultConstructor() {
    FormatVersionNormalizedEvent underTest = new FormatVersionNormalizedEvent();
    assertThat(underTest.getFormat(), is(nullValue()));
  }

  @Test(expected = NullPointerException.class)
  public void testNullFormatRejected() {
    new FormatVersionNormalizedEvent(null);
  }

  @Test
  public void testDifferentFormats() {
    assertThat(new FormatVersionNormalizedEvent("npm").getFormat(), is("npm"));
    assertThat(new FormatVersionNormalizedEvent("docker").getFormat(), is("docker"));
    assertThat(new FormatVersionNormalizedEvent("nuget").getFormat(), is("nuget"));
  }
}
