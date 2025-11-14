
package org.globalart.astrum.systemchecks;

import java.util.stream.Stream;

/**
 * Service providing access to the results of System checks for Astrum instances.
 */
public interface SystemCheckService
{
  /**
   * Return the available system check results
   */
  Stream<NodeSystemCheckResult> getResults();
}
