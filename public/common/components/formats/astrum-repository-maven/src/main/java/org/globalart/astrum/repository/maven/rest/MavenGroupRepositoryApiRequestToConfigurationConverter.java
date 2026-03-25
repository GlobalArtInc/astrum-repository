
package org.globalart.astrum.repository.maven.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.GroupRepositoryApiRequestToConfigurationConverter;
import org.springframework.stereotype.Component;

/**
 * Converter for Maven group repository API requests to Configuration.
 *
 * @since 3.77
 */
@Component
public class MavenGroupRepositoryApiRequestToConfigurationConverter
    extends GroupRepositoryApiRequestToConfigurationConverter<MavenGroupRepositoryApiRequest>
{
  @Override
  public Configuration convert(final MavenGroupRepositoryApiRequest request) {
    return super.convert(request);
  }
}
