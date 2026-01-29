
package org.globalart.astrum.repository;

import static org.apache.commons.lang3.StringUtils.isEmpty;

public class ETagHeaderUtils
{
  public static final String WEAK_DESIGNATOR = "W/";

  private ETagHeaderUtils() {
  }

  /**
   * Adds quotes to etag header per spec.
   * https://tools.ietf.org/html/rfc7232#section-2.3
   */
  public static String quote(final String etag) {
    if (etag.startsWith(WEAK_DESIGNATOR)) {
      return etag;
    }
    else {
      return "\"" + etag + "\"";
    }
  }

  /**
   * Removes quotes from etag header.
   */
  public static String extract(final String etag) {
    if (!isEmpty(etag) && etag.startsWith("\"") && etag.endsWith("\"")) {
      return etag.substring(1, etag.length() - 1);
    }
    else {
      return etag;
    }
  }
}
