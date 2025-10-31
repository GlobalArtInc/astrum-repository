
package org.globalart.astrum.common.collect;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for {@link StringMultimap}.
 */
public class StringMultimapTest

{
  private StringMultimap underTest;

  @Before
  public void setUp() {
    underTest = new StringMultimap();
  }

  @Test
  public void testSizeValidation() {
    assertThat(underTest.size(), is(0));

    underTest.set("foo", "bar");
    assertThat(underTest.size(), is(1));

    underTest.set("foo", "baz");
    assertThat(underTest.size(), is(1));

    underTest.set("ick", "poo");
    assertThat(underTest.size(), is(2));
  }
}
