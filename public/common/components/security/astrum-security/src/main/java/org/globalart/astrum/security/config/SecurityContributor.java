
package org.globalart.astrum.security.config;

/**
 * Allows contribution to security model.
 *
 * @since 3.1
 */
public interface SecurityContributor
{
  /**
   * Gets the security contribution.
   */
  SecurityConfiguration getContribution();
}
