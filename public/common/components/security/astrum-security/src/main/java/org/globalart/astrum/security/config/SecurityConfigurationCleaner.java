
package org.globalart.astrum.security.config;

/**
 * Will handle cleaning existing configuration when an item is removed
 */
public interface SecurityConfigurationCleaner
{
  /**
   * Called when a role is removed so additional cleanup logic can be done.
   */
  void roleRemoved(SecurityConfiguration configuration, String roleId);

  /**
   * Called when a privilege is removed so additional cleanup logic can be done.
   */
  void privilegeRemoved(SecurityConfiguration configuration, String privilegeId);
}
