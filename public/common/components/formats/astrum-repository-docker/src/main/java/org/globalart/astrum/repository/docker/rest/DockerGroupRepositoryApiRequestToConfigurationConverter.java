package org.globalart.astrum.repository.docker.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.rest.GroupRepositoryApiRequestToConfigurationConverter;

import org.springframework.stereotype.Component;

@Component
public class DockerGroupRepositoryApiRequestToConfigurationConverter
    extends GroupRepositoryApiRequestToConfigurationConverter<DockerGroupRepositoryApiRequest>
{
  @Override
  public Configuration convert(final DockerGroupRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    DockerAttributes.apply(request.getDocker(), configuration.attributes(DockerConfig.DOCKER));
    return configuration;
  }
}
