
package org.globalart.astrum.validation.ssrf;

import jakarta.validation.ValidationException;

/**
 * Service for SSRF protection validation and configuration management.
 */
public interface AntiSsrfService
{
  /**
   * Validates a hostname or IP address. Results are cached.
   *
   * @param host the hostname or IP address to validate
   * @throws ValidationException if the host is blocked by SSRF protection
   */
  void validateHost(String host);

  /**
   * Validates a hostname or IP address without caching, for use during repository or blob-store
   * configuration saves where a fresh evaluation against the current config is needed.
   *
   * @param host the hostname or IP address to validate
   * @throws ValidationException if the host is blocked by SSRF protection
   */
  void validateHostWithoutCache(String host);

  /**
   * Returns the current SSRF protection configuration.
   */
  SsrfProtectionConfiguration getConfiguration();

  /**
   * Updates the SSRF protection configuration.
   *
   * @param configuration the new configuration to apply
   */
  void updateConfiguration(SsrfProtectionConfiguration configuration);
}
