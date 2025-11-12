
package org.globalart.astrum.internal.node;

import java.util.Optional;

/**
 * Designed to retrieve deployment id (Astrum nodeId from Orient)
 */
public interface DeploymentIdStore
{
  /**
   * Retrieving deployment id
   *
   * @return String representation of deployment id
   */
  Optional<String> get();

  void set(String deploymentId);
}
