
package org.globalart.astrum.bootstrap.entrypoint.edition;

/**
 * List of the edition priorities. Using 100 spacing to allow future insertions without modifying all lines.
 */
public final class AstrumEditionPriority
{
  // Cloud edition will always take priority and be checked first
  public static final int CLOUD_PRIORITY = Integer.MIN_VALUE;

  public static final int PRO_PRIORITY = CLOUD_PRIORITY + 100;

  public static final int COMMUNITY_PRIORITY = PRO_PRIORITY + 100;

  // Core is the last edition to be checked
  public static final int CORE_PRIORITY = COMMUNITY_PRIORITY + 100;
}
