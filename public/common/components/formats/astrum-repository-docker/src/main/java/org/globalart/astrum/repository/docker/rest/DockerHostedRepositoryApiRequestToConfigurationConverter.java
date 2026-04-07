package org.globalart.astrum.repository.docker.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationConstants;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.rest.api.HostedRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.rest.api.model.DockerHostedStorageAttributes;

import org.springframework.stereotype.Component;

@Component
public class DockerHostedRepositoryApiRequestToConfigurationConverter
    extends HostedRepositoryApiRequestToConfigurationConverter<DockerHostedRepositoryApiRequest>
{
  @Override
  public Configuration convert(final DockerHostedRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    DockerAttributes.apply(request.getDocker(), configuration.attributes(DockerConfig.DOCKER));
    if (request.getStorage() instanceof DockerHostedStorageAttributes storage && storage.getLatestPolicy() != null) {
      configuration.attributes(ConfigurationConstants.STORAGE)
          .set(ConfigurationConstants.LATEST_POLICY, storage.getLatestPolicy());
    }
    return configuration;
  }
}
