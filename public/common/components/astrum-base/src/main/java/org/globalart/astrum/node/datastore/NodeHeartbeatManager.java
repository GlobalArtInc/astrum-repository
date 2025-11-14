
package org.globalart.astrum.node.datastore;

import java.util.Collection;
import java.util.Map;
import java.util.stream.Stream;

import org.globalart.astrum.systemchecks.NodeSystemCheckResult;

/**
 * Create the job which will collect information about nodes and periodically write it to DB.
 */
public interface NodeHeartbeatManager
{
  String VERSION_ATTRIBUTE = "version";

  String CLUSTERED_ATTRIBUTE = "clustered";

  String NODE_ID = "nodeId";

  String CACHE_KEY = "nodes";

  /**
   * Determines if the current deployment is on sync with their database/deployment type
   *
   * @return {@code true} in case of a valid deployment, otherwise {@code false}.
   */
  boolean isValidNodeDeployment();

  /**
   * Get the {@link NodeSystemCheckResult} for the active nodes
   */
  Stream<NodeSystemCheckResult> getSystemChecks();

  /**
   * Determines if the current node is in a clustered mode
   */
  boolean isCurrentNodeClustered();

  /**
   * Triggers a write of the latest heartbeat information
   */
  void writeHeartbeat();

  /**
   * Collects and transforms system info from heartbeat table
   */
  Map<String, Map<String, Object>> getSystemInformationForNodes();

  /**
   * Collects nodeInfo from the heartbeat table
   */
  Collection<NodeHeartbeat> getActiveNodeHeartbeatData();
}
