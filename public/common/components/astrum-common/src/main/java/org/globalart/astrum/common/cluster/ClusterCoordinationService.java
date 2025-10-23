
package org.globalart.astrum.common.cluster;

import java.time.Duration;

/**
 * Service for coordinating exclusive access to cluster-wide resources.
 *
 * This service provides distributed locking capabilities that work across multiple nodes in a cluster.
 * In single-node deployments, the default implementation provides local (JVM-level) coordination.
 */
public interface ClusterCoordinationService
{
  /**
   * Attempts to acquire exclusive access to a cluster-wide resource.
   *
   * This method will attempt to acquire a distributed lock for the specified resource. If another node
   * in the cluster already holds the lock, this method will return a {@link ClusterLock} where
   * {@link ClusterLock#isAcquired()} returns false, indicating the lock is held by another node.
   *
   * The lock will automatically expire after the specified duration to prevent deadlocks in case
   * the acquiring node crashes or fails to release the lock explicitly.
   *
   * @param resourceKey unique identifier for the resource (e.g., "docker-gc-my-repo")
   * @param duration how long the lock should be held before automatically expiring
   * @return a {@link ClusterLock} indicating whether the lock was acquired and which node owns it
   */
  ClusterLock acquireResourceLock(String resourceKey, Duration duration);

  /**
   * Releases a previously acquired lock for a cluster-wide resource.
   *
   * This should be called in a finally block to ensure locks are released even if exceptions occur.
   * If the current node does not own the lock, this method will have no effect.
   *
   * @param resourceKey the unique identifier for the resource to release
   */
  void releaseResourceLock(String resourceKey);

  /**
   * Represents the result of a lock acquisition attempt.
   *
   * This interface provides information about whether the lock was successfully acquired
   * and which node currently owns the lock.
   */
  interface ClusterLock
  {
    /**
     * Indicates whether the current node successfully acquired the lock.
     *
     * @return true if the current node owns the lock, false if another node holds it
     */
    boolean isAcquired();

    /**
     * Returns the identifier of the node that currently owns the lock.
     *
     * This can be used for logging and diagnostics to identify which node is holding the lock.
     * In single-node deployments, this typically returns a local identifier like "local".
     *
     * @return the node ID that owns the lock
     */
    String getOwnerNodeId();

    /**
     * Returns the resource key associated with this lock.
     *
     * @return the resource key
     */
    String getResourceKey();
  }
}
