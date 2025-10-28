

package org.globalart.astrum.common.node;

import javax.annotation.Nullable;

import org.globalart.astrum.common.lifecycle.Lifecycle;

/**
 * Provides access to managing deployment identity.
 *
 * {@link #getId()} is uniquely generated at first run and is immutable. In non-clustered deployments,
 * this value is likely to be the same as {@link NodeAccess#getId()}.
 *
 * In clustered environments however, {@link #getId()} will be the same across all nodes, where as
 * node identity ({@link NodeAccess#getId()}) will differ from node to node.
 *
 * @since 3.6.1
 */
public interface DeploymentAccess
    extends Lifecycle
{
  /**
   * @return the String id (never null)
   */
  String getId();

  /**
   * Alias is intended to be a user-provided identity for this deployment, similar to friendly node name.
   * Like {@link #getId()}, the value returned by this method will be consistent across all nodes in a cluster.
   *
   * @return the String id, or null if not set
   */
  @Nullable
  String getAlias();

  /**
   * @param newAlias the new alias for this deployment
   */
  void setAlias(@Nullable String newAlias);
}
