
package org.globalart.astrum.httpclient.config;

import java.util.Set;

import org.junit.Test;
import org.mockito.Mock;

import jakarta.validation.ConstraintValidatorContext;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests for {@link NonProxyHostsValidator}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class NonProxyHostsValidatorTest
{

  @Mock
  private ConstraintValidatorContext context;

  private NonProxyHostsValidator validator = new NonProxyHostsValidator();

  private void validateAndExpect(String expression, boolean expected) {
    assertThat(validator.isValid(Set.of(expression), context), equalTo(expected));
  }

  @Test
  public void validationPositiveTest() {
    validateAndExpect("globalart.dev", true);
    validateAndExpect("*.globalart.dev", true);
    validateAndExpect("*.globalart.*", true);
    validateAndExpect("1.2.3.4", true);
    validateAndExpect("*.2.3.4", true);
    validateAndExpect("1.2.3.*", true);
    validateAndExpect("*.2.3.*", true);
    validateAndExpect("10.*", true);
    validateAndExpect("*.10", true);
    validateAndExpect("csétamás.hu", true);
    validateAndExpect("2001:db8:85a3:8d3:1319:8a2e:370:7348", true);
    validateAndExpect("[2001:db8:85a3:8d3:1319:8a2e:370:7348]", true);
    validateAndExpect("[::1]", true);
    validateAndExpect("*:8]", true);
    validateAndExpect("[:*", true);
    validateAndExpect("localhost", true);
  }

  @Test
  public void validationNegativeTest() {
    // these below are the "best effort" we can rule out
    validateAndExpect("", false);
    validateAndExpect("  ", false);
    validateAndExpect("foo|globalart.dev", false);
    validateAndExpect("comma,com", false);
    validateAndExpect("[*:8]", false);
  }
}
