
package org.globalart.astrum.common.text;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

/**
 * Tests for {@link Plural}
 */
public class PluralTest

{
  @Test
  public void testSimplePlural() {
    assertThat(Plural.of(-1, "dog"), is("-1 dogs"));
    assertThat(Plural.of(0, "dog"), is("0 dogs"));
    assertThat(Plural.of(1, "dog"), is("1 dog"));
    assertThat(Plural.of(2, "dog"), is("2 dogs"));
  }

  @Test
  public void testComplexPlural() {
    assertThat(Plural.of(-1, "candy", "candies"), is("-1 candies"));
    assertThat(Plural.of(0, "candy", "candies"), is("0 candies"));
    assertThat(Plural.of(1, "candy", "candies"), is("1 candy"));
    assertThat(Plural.of(2, "candy", "candies"), is("2 candies"));
  }
}
