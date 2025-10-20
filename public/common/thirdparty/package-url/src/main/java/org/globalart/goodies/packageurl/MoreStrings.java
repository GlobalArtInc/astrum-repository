
package org.globalart.goodies.packageurl;

import java.util.ArrayList;
import java.util.List;
import java.util.Locale;

import javax.annotation.Nullable;

/**
 * String helpers.
 *
 * @since 1.1.0
 */
final class MoreStrings
{
  private MoreStrings() {
    // empty
  }

  /**
   * Convert given value to lower-case.
   */
  static String lowerCase(final String value) {
    return value.toLowerCase(Locale.ENGLISH);
  }

  /**
   * Convert given list of values to lower-case.
   */
  @Nullable
  static List<String> lowerCase(@Nullable final List<String> values) {
    if (values != null) {
      List<String> result = new ArrayList<>(values.size());
      for (String value : values) {
        result.add(lowerCase(value));
      }
      return result;
    }
    return null;
  }
}
