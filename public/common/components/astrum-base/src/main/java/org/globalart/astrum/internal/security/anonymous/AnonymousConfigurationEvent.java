
package org.globalart.astrum.internal.security.anonymous;

import org.globalart.astrum.security.anonymous.AnonymousConfiguration;

/**
 * {@link AnonymousConfiguration} event.
 *
 * @since 3.2
 */
public interface AnonymousConfigurationEvent
{
  boolean isLocal();

  String getRemoteNodeId();

  /**
   * @since 3.10
   */
  AnonymousConfiguration getAnonymousConfiguration();
}
