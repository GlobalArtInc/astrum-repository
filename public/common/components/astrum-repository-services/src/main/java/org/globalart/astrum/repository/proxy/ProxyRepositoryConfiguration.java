
package org.globalart.astrum.repository.proxy;

import java.net.URI;
import java.time.Duration;

import org.globalart.astrum.repository.types.ProxyType;

/**
 * Configuration object for {@link ProxyType} repositories
 */
public interface ProxyRepositoryConfiguration
{
  /**
   * Content max-age.
   */
  Duration getContentMaxAge();

  /**
   * Metadata max-age.
   */
  Duration getMetadataMaxAge();

  /**
   * The remote URI of the proxy repository.
   */
  URI getRemoteURL();
}
