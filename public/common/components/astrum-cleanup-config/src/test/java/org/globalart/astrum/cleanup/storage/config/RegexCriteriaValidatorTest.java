
package org.globalart.astrum.cleanup.storage.config;

import org.globalart.astrum.cleanup.storage.config.RegexCriteriaValidator.InvalidExpressionException;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;

public class RegexCriteriaValidatorTest

{
  private static final String EXPRESSION = "org/globalart";

  @Test(expected = InvalidExpressionException.class)
  public void whenInvalidExpressionExceptionThrown() {
    RegexCriteriaValidator.validate("hello(world");
  }

  @Test
  public void whenValidExpressionQueryIsBuilt() {
    assertThat(RegexCriteriaValidator.validate(EXPRESSION), is(EXPRESSION));
  }
}
