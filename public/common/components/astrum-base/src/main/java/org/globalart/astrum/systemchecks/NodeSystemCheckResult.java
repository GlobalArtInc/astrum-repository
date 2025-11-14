
package org.globalart.astrum.systemchecks;

import java.util.Map;

import com.codahale.metrics.health.HealthCheck.Result;

/**
 * System check results.
 */
public class NodeSystemCheckResult
{
  private final String hostname;

  private final String nodeId;

  private final Map<String, Result> systemChecks;

  public NodeSystemCheckResult(
      final String nodeId,
      final String hostname,
      final Map<String, Result> systemChecks)
  {
    this.nodeId = nodeId;
    this.hostname = hostname;
    this.systemChecks = systemChecks;
  }

  /**
   * The nodeId of the system which generated this result
   */
  public String getNodeId() {
    return nodeId;
  }

  /**
   * The hostname of the node which generated this result
   */
  public String getHostname() {
    return hostname;
  }

  /**
   * The system check results for the node
   */
  public Map<String, Result> getResult() {
    return systemChecks;
  }
}
