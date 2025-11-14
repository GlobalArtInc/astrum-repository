
package org.globalart.astrum.node.datastore;

import java.time.OffsetDateTime;
import java.util.Map;

import org.globalart.astrum.common.entity.ContinuationAware;

/**
 * Heartbeat of a node pinging the database indicating its presence.
 *
 * @since 3.37
 */
public interface NodeHeartbeat
    extends ContinuationAware
{
  /**
   * The time of the last heartbeat
   */
  OffsetDateTime heartbeatTime();

  /**
   * Host name derived for the local system
   */
  String hostname();

  /**
   * The database record ID
   */
  Integer id();

  Map<String, Object> nodeInfo();

  Map<String, Object> systemInfo();

  /**
   * The Heartbeat ID of the node, this is a non-persistent value that is distinct from the NodeAccess ID. Only useful
   * to
   * determine whether the heartbeat was created while an instance is running.
   */
  String heartbeatId();

  void setHeartbeatTime(final OffsetDateTime heartbeatTime);

  void setHostname(final String hostname);

  void setId(final Integer id);

  void setHeartbeatId(final String heartbeatId);

  void setNodeInfo(final Map<String, Object> nodeInfo);

  void setSystemInfo(final Map<String, Object> systemInfo);
}
