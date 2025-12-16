
package org.globalart.astrum.security.anonymous;

import org.apache.shiro.subject.Subject;

/**
 * Anonymous manager.
 *
 * @since 3.0
 */
public interface AnonymousManager
{
  /**
   * Returns copy of current anonymous configuration.
   */
  AnonymousConfiguration getConfiguration();

  /**
   * Provide a new instance of an AnonymousConfiguration.
   *
   * @since 3.20
   */
  AnonymousConfiguration newConfiguration();

  /**
   * Installs new anonymous configuration.
   */
  void setConfiguration(AnonymousConfiguration configuration);

  /**
   * Check if anonymous access is enabled.
   */
  boolean isEnabled();

  /**
   * Build anonymous subject.
   */
  Subject buildSubject();

  /**
   * Check whether the system has been configured
   *
   * @since 3.17
   */
  boolean isConfigured();
}
