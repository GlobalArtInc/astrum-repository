
package org.globalart.astrum.security.config;

import java.util.Set;

/**
 * Source of {@link SecurityConfiguration}.
 */
public interface SecurityConfigurationSource
{
  /**
   * The default user id for the admin user.
   */
  public static final String ADMIN = "admin";

  /**
   * The default user id for the anonymous user.
   */
  public static final String ANONYMOUS = "anonymous";

  /**
   * Gets the current configuration.
   *
   * @return the configuration, null if not loaded
   */
  SecurityConfiguration getConfiguration();

  /**
   * Gets the current configuration for specified users
   *
   * @return the configuration, null if not loaded
   */
  default SecurityConfiguration getConfiguration(Set<String> userIds) {
    return getConfiguration();
  }

  /**
   * Forces reloading the user configuration.
   *
   * @return the configuration
   */
  SecurityConfiguration loadConfiguration();
}
