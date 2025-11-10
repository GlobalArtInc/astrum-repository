
package org.globalart.astrum.common.app;

import org.springframework.core.Ordered;

/**
 * List of the filter priorities. Please field order by priority, 100 spacing used to allow future insertions without
 * modifying all lines.
 */
public final class WebFilterPriority
{
  /**
   * A filter which provides the servlet to the thread
   */
  public static final int SERVLET_FILTER = Ordered.HIGHEST_PRECEDENCE;

  /**
   * Filters providing authentication
   */
  public static final int AUTHENTICATION = 0;

  /**
   * Anti-CSRF protection, must be after {@link AUTHENTICATION} as auth is used as a heuristic.
   */
  public static final int ANTI_CSRF = AUTHENTICATION + 100;

  public static final int WEB = -0x70000000;

  public static final int LICENSING = -0x60000000;

  public static final int LEGACY_HTTP_BRIDGE = -0x50000000;

  public static final int WEB_RESOURCES = Ordered.LOWEST_PRECEDENCE;

  private WebFilterPriority() {
    // prevent creation
  }
}
