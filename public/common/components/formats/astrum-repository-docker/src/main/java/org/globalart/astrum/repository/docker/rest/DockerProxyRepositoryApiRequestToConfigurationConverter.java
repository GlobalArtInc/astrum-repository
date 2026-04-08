package org.globalart.astrum.repository.docker.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class DockerProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<DockerProxyRepositoryApiRequest>
{
  @Autowired
  public DockerProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public Configuration convert(final DockerProxyRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    DockerAttributes.apply(request.getDocker(), configuration.attributes(DockerConfig.DOCKER));
    DockerProxyAttributes.apply(request.getDockerProxy(), configuration.attributes(DockerConfig.DOCKER_PROXY));
    return configuration;
  }
}
