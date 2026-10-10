
package org.globalart.astrum.validation.constraint;

/**
 * Constants for validating that names for entities in the system fall within defined limits.
 *
 * @since 3.0
 */
public final class NamePatternConstants
{
  // Changes to this constant should be sync'd in:
  // components/astrum-rapture/src/main/resources/static/rapture/AS/util/Validator.js
  public static final String REGEX = "^[a-zA-Z0-9\\-]{1}[a-zA-Z0-9_\\-\\.]*$";

  public static final String REGEX_WITH_WILDCARDS = "^[a-zA-Z0-9\\-]{1}[a-zA-Z0-9_\\-\\*\\.]*$";

  public static final String MESSAGE = "{org.globalart.astrum.validation.constraint.name}";

  private NamePatternConstants() {
  }
}
