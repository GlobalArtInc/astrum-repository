
package org.globalart.astrum.common.upgrade;

/**
 * Upgrades a specific model/schema from one version to another.
 * 
 * @since 3.1
 */
public interface Upgrade
{
  /**
   * Attempts to apply the upgrade to the current installation.
   */
  void apply() throws Exception;
}
