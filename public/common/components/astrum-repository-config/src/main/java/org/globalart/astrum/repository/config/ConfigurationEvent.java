
package org.globalart.astrum.repository.config;

import org.globalart.astrum.common.event.HasLocality;

/**
 * Repository {@link Configuration} event.
 *
 * @since 3.1
 */
public interface ConfigurationEvent
    extends HasLocality
{
  @Override
  boolean isLocal();

  String getRepositoryName();

  Configuration getConfiguration();
}
