
package org.globalart.astrum.common.time;

import java.text.DateFormat;
import java.text.ParseException;
import java.text.SimpleDateFormat;
import java.util.Date;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Helper for working with <a href="http://en.wikipedia.org/wiki/ISO_8601">ISO 8601<a/> dates.
 *
 */
public class Iso8601Date
{
  public static final String PATTERN = "yyyy-MM-dd'T'HH:mm:ss.SSSXXX"; // NON-NLS
  // public static final String PATTERN = "yyyy-MM-dd'T'HH:mm:ss";

  /**
   */
  private Iso8601Date() {
  }

  private static DateFormat getFormat() {
    return new SimpleDateFormat(PATTERN);
  }

  public static Date parse(final String value) throws ParseException {
    checkNotNull(value);
    return getFormat().parse(value);
  }

  public static String format(final Date date) {
    checkNotNull(date);
    return getFormat().format(date);
  }
}
