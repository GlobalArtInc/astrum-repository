
package org.globalart.astrum.common.atlas;

import java.util.Map;

/**
 * Generates system information report.
 *
 * @since 2.7
 */
public interface SystemInformationGenerator
{
  /**
   * Generates a tree-structured report of critical system information details.
   */
  Map<String, Object> report();
}
