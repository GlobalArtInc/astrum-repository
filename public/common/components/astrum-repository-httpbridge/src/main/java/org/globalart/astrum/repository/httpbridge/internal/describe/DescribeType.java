
package org.globalart.astrum.repository.httpbridge.internal.describe;

import org.globalart.astrum.common.text.Strings2;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Describe type.
 *
 * @since 3.0
 */
public enum DescribeType
{
  HTML,
  JSON;

  /**
   * Parse type from flags. Either explicit HTML or JSON, or anything else will default to HTML.
   */
  public static DescribeType parse(String flags) {
    checkNotNull(flags);
    flags = Strings2.upper(flags).trim();
    if (flags.isEmpty()) {
      return HTML;
    }
    try {
      return valueOf(flags);
    }
    catch (IllegalArgumentException e) {
      return HTML;
    }
  }
}
