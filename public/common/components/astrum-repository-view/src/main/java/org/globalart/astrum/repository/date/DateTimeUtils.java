
package org.globalart.astrum.repository.date;

import org.joda.time.DateTime;
import org.joda.time.format.DateTimeFormatter;

import static java.util.Locale.US;
import static org.joda.time.DateTimeZone.forID;
import static org.joda.time.format.DateTimeFormat.forPattern;

/**
 * Utils for the astrum repository allowing simplified access to Date and Time methods for easy reuse.
 *
 * @since 3.16
 */
public class DateTimeUtils
{
  /**
   * Date format pattern used to parse HTTP date headers in RFC 1123 format.
   */
  private static final String PATTERN_RFC1123 = "EEE, dd MMM yyyy HH:mm:ss zzz";

  public static final DateTimeFormatter FORMATTER_RFC1123 = forPattern(PATTERN_RFC1123)
      .withZone(forID("GMT"))
      .withLocale(US);

  private DateTimeUtils() {
  }

  /**
   * Formats the given {@link DateTime} according to the RFC 1123 pattern.
   *
   * @param dateTime The date to format.
   * @return An RFC 1123 formatted date string.
   * @see #PATTERN_RFC1123
   */
  public static String formatDateTime(final DateTime dateTime) {
    return FORMATTER_RFC1123.print(dateTime);
  }
}
