
package org.globalart.astrum.common.jdbc;

import java.util.regex.Matcher;
import java.util.regex.Pattern;

import static java.util.regex.Pattern.compile;

/**
 * Offers static helper methods for common operations to hide sensitive data in the jdbcUrl.
 *
 * @since 3.36
 */
public final class JdbcUrlRedactor
{
  private static final Pattern passwordPattern = compile("password=(.*?)(?=&\\w*=)|password=(.*)");

  public static final String REDACTED = "**REDACTED**";

  private JdbcUrlRedactor() {
    // empty
  }

  /**
   * Hide sensitive data in the jdbcUrl
   *
   * @param jdbcUrl original url
   * @return redacted url
   */
  public static String redactPassword(final String jdbcUrl) {
    String result = jdbcUrl;
    Matcher matcher = passwordPattern.matcher(jdbcUrl);
    while (matcher.find()) {
      String password = matcher.group(1) != null ? matcher.group(1) : matcher.group(2);
      result = result.replaceFirst("password=" + password, "password=" + REDACTED);
    }
    return result;
  }
}
