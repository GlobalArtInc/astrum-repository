
package org.globalart.astrum.node.upgrade;

import java.util.Map;
import java.util.Objects;
import javax.annotation.Nullable;

import org.globalart.astrum.node.datastore.NodeHeartbeat;
import org.globalart.astrum.node.datastore.NodeHeartbeatManager;
import org.globalart.astrum.scheduling.TaskSupport;

/**
 * Base class for migration tasks.
 */
public abstract class MigrationTaskSupport
    extends TaskSupport
{
  private static final String ASTRUM_STATUS_KEY = "astrum-status";

  private static final String VERSION_KEY = "version";

  private static final String BUILD_REVISION_KEY = "buildRevision";

  protected final NodeHeartbeatManager heartbeatManager;

  public MigrationTaskSupport(
      @Nullable final NodeHeartbeatManager heartbeatManager)
  {
    this.heartbeatManager = heartbeatManager;
  }

  /**
   * Checks if any nodes in the cluster are running different versions, Returns false in OSS/single-node
   * deployments (no HA).
   *
   * @return true if any nodes have different version/buildRevision, false otherwise
   */
  protected boolean oldNodesRunning() {
    if (heartbeatManager == null) {
      return false;
    }

    return heartbeatManager.getActiveNodeHeartbeatData()
        .stream()
        .map(NodeHeartbeat::systemInfo)
        .filter(Objects::nonNull)
        .map(systemInfo -> systemInfo.get(ASTRUM_STATUS_KEY))
        .filter(Objects::nonNull)
        .filter(astrumStatus -> astrumStatus instanceof Map)
        .map(astrumStatus -> (Map<String, String>) astrumStatus)
        .map(astrumStatus -> {
          String version = astrumStatus.get(VERSION_KEY);
          String buildRevision = astrumStatus.get(BUILD_REVISION_KEY);
          return version + "|" + buildRevision;
        })
        .distinct()
        .count() > 1L;
  }
}
