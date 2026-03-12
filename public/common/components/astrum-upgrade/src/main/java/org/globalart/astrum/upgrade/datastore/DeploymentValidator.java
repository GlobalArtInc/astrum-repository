
package org.globalart.astrum.upgrade.datastore;

/**
 * Validates that the current deployment is validate before starting
 */
public interface DeploymentValidator
{
  /**
   * Validates that the system
   */
  void validate();
}
