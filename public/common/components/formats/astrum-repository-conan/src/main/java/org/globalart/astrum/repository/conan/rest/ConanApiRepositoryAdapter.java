package org.globalart.astrum.repository.conan.rest;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.conan.internal.proxy.ConanProxyFacet;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(ConanFormat.NAME)
public class ConanApiRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  @Autowired
  public ConanApiRepositoryAdapter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    if (!ProxyType.NAME.equals(repository.getType().toString())) {
      return super.adapt(repository);
    }
    String version = repository.getConfiguration()
        .attributes(ConanProxyFacet.CONFIG_KEY)
        .get(ConanProxyFacet.CONAN_VERSION, String.class);
    return new ConanProxyApiRepository(
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
        new ConanProxyAttributes(ConanProxyFacet.normalizeVersion(version)));
  }
}
