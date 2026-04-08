package org.globalart.astrum.repository.docker.rest;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.ConfigurationConstants;
import org.globalart.astrum.repository.docker.DockerConfig;
import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.rest.api.model.DockerHostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(DockerFormat.NAME)
public class DockerRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  @Autowired
  public DockerRepositoryAdapter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    DockerAttributes docker = DockerAttributes.from(repository.getConfiguration().attributes(DockerConfig.DOCKER));
    switch (repository.getType().toString()) {
      case HostedType.NAME:
        return new DockerHostedApiRepository(
            repository.getName(),
            repository.getUrl(),
            repository.getConfiguration().isOnline(),
            dockerHostedStorage(repository),
            getCleanupPolicyAttributes(repository),
            getComponentAttributes(repository),
            docker);
      case ProxyType.NAME:
        return new DockerProxyApiRepository(
            repository.getName(),
            repository.getUrl(),
            repository.getConfiguration().isOnline(),
            getStorageAttributes(repository),
            getCleanupPolicyAttributes(repository),
            getProxyAttributes(repository),
            getNegativeCacheAttributes(repository),
            getHttpClientAttributes(repository),
            getRoutingRuleName(repository),
            getReplicationAttributes(repository),
            docker,
            DockerProxyAttributes.from(repository.getConfiguration().attributes(DockerConfig.DOCKER_PROXY)));
      case GroupType.NAME:
        return new DockerGroupApiRepository(
            repository.getName(),
            repository.getUrl(),
            repository.getConfiguration().isOnline(),
            getStorageAttributes(repository),
            getGroupAttributes(repository),
            docker);
      default:
        return super.adapt(repository);
    }
  }

  private DockerHostedStorageAttributes dockerHostedStorage(final Repository repository) {
    HostedStorageAttributes storage = getHostedStorageAttributes(repository);
    NestedAttributesMap attributes = repository.getConfiguration().attributes(ConfigurationConstants.STORAGE);
    Object latest = attributes.get(ConfigurationConstants.LATEST_POLICY);
    return new DockerHostedStorageAttributes(storage.getBlobStoreName(), storage.getStrictContentTypeValidation(),
        storage.getWritePolicy(), latest == null ? null : DockerConfig.bool(latest, false));
  }
}
