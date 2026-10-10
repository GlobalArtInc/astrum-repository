
package org.globalart.astrum.security.config;

import org.apache.commons.lang3.StringEscapeUtils;

public final class PrivilegeUtils
{
  /**
   * Matches any invalid character or one or more of '_' or '.' at the beginning of the string (which could be
   * prefixed by invalid chars).
   */
  private static final String NAME_INVALID_CHAR_REGEX = "^[^a-zA-Z0-9\\-]+|[^a-zA-Z0-9_\\-\\.]";

  private PrivilegeUtils() {
  }

  /**
   * Decode, then replace any spaces with underscore, then clean any invalid characters,
   * and finally clean any trailing underscores.
   */
  public static String cleanName(final String name) {
    if (name != null) {
      return StringEscapeUtils.unescapeHtml4(name).trim().replaceAll(" ", "_").replaceAll(NAME_INVALID_CHAR_REGEX, "");
    }

    return name;
  }
}
