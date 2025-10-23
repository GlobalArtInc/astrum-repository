
package org.globalart.astrum.common.cluster;

import java.time.Duration;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Default implementation of {@link ClusterCoordinationService} for single-node deployments.
 *
 * This implementation provides local (JVM-level) coordination and always succeeds in acquiring locks
 * since there are no other nodes in the cluster to coordinate with.
 *
 * This bean is automatically used when no other {@link ClusterCoordinationService} implementation is available,
 * such as in Core Edition or when clustering is disabled.
 *
 * @since 3.87
 */
public class LocalClusterCoordinationService
    implements ClusterCoordinationService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String LOCAL_NODE_ID = "local";

  @Override
  public ClusterLock acquireResourceLock(final String resourceKey, final Duration duration) {
    log.debug("Local lock acquired for resource: {} (duration: {})", resourceKey, duration);
    return new LocalClusterLock(resourceKey, LOCAL_NODE_ID);
  }

  @Override
  public void releaseResourceLock(final String resourceKey) {
    log.debug("Local lock released for resource: {}", resourceKey);
  }

  /**
   * Local implementation of {@link ClusterLock} that always indicates successful acquisition.
   */
  private static class LocalClusterLock
      implements ClusterLock
  {
    private final String resourceKey;

    private final String ownerNodeId;

    LocalClusterLock(final String resourceKey, final String ownerNodeId) {
      this.resourceKey = resourceKey;
      this.ownerNodeId = ownerNodeId;
    }

    @Override
    public boolean isAcquired() {
      return true; // Always acquired in single-node mode
    }

    @Override
    public String getOwnerNodeId() {
      return ownerNodeId;
    }

    @Override
    public String getResourceKey() {
      return resourceKey;
    }
  }
}
