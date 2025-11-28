
package org.globalart.astrum.validation.constraint;

/**
 * Constants for validating logger names in the system.
 *
 */

public final class LoggerNameValidator
{

  private LoggerNameValidator() {
    // utility class
  }

  /**
   * Regex pattern for valid logger names.
   * Matches Java package names and ROOT logger.
   */
  public static final String REGEX = ".*[<>&'\"\\n\\r\\t/].*";

  /**
   * Error message for invalid logger names.
   */
  public static final String MESSAGE =
      "Logger name cannot include <, >, &, ', \", /, newline, or tab characters";

  /**
   * @param name the logger name to validate
   * @throws IllegalArgumentException if the name is invalid
   */
  public static void validate(String name) throws IllegalArgumentException {
    if (name == null || name.trim().isEmpty()) {
      throw new IllegalArgumentException("Logger name is required");
    }

    if (name.matches(LoggerNameValidator.REGEX)) {
      throw new IllegalArgumentException(LoggerNameValidator.MESSAGE);
    }
  }
}
