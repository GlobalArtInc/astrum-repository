
package org.globalart.astrum.internal.security.anonymous;

import javax.annotation.Nullable;

import org.globalart.astrum.security.anonymous.AnonymousConfiguration;

/**
 * {@link AnonymousConfiguration} store.
 *
 * @since 3.0
 */
public interface AnonymousConfigurationStore
{
  // TODO: Sort out exceptions, both of these should have some expected exceptions

  @Nullable
  AnonymousConfiguration load();

  void save(AnonymousConfiguration configuration);

  /**
   * Provide a new instance of {@link ApplicationConfiguration} applicable for use with this backing store.
   *
   * @since 3.20
   */
  AnonymousConfiguration newConfiguration();
}
