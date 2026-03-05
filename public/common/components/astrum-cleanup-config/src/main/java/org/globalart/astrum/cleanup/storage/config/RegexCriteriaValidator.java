
package org.globalart.astrum.cleanup.storage.config;

import java.util.regex.Pattern;

import static java.lang.String.format;

/**
 * @since 3.19
 */
public class RegexCriteriaValidator
{
  private RegexCriteriaValidator() {
  }

  /**
   * Ensures that a regular expression entered is a valid pattern.
   *
   * @param expression
   * @throws InvalidExpressionException when the expression is deemed invalid
   */
  public static String validate(final String expression) {
    try {
      Pattern.compile(expression);
    }
    catch (Exception e) { // NOSONAR
      throw new InvalidExpressionException(
          format("Invalid regular expression pattern: %s", e.getMessage()));
    }
    return expression;
  }

  public static class InvalidExpressionException
      extends RuntimeException
  {
    InvalidExpressionException(final String message) {
      super(message);
    }
  }
}
