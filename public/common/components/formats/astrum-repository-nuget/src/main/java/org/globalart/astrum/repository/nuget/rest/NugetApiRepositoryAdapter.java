package org.globalart.astrum.repository.nuget.rest;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.nuget.internal.proxy.NugetProxyFacet;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.nuget.rest.NugetAttributes.DEFAULT_QUERY_CACHE_ITEM_MAX_AGE;
import static org.globalart.astrum.repository.nuget.rest.NugetAttributes.NUGET_VERSION;
import static org.globalart.astrum.repository.nuget.rest.NugetAttributes.QUERY_CACHE_ITEM_MAX_AGE;

@Component
@Qualifier(NugetFormat.NAME)
public class NugetApiRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  @Autowired
  public NugetApiRepositoryAdapter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    if (!ProxyType.NAME.equals(repository.getType().toString())) {
      return super.adapt(repository);
    }
    return new NugetProxyApiRepository(
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
        nugetAttributes(repository));
  }

  private static NugetAttributes nugetAttributes(final Repository repository) {
    NestedAttributesMap attributes = repository.getConfiguration().attributes(NugetProxyFacet.CONFIG_KEY);
    Number maxAge = attributes.get(QUERY_CACHE_ITEM_MAX_AGE, Number.class);
    String version = attributes.get(NUGET_VERSION, String.class);
    return new NugetAttributes(maxAge != null ? maxAge.intValue() : DEFAULT_QUERY_CACHE_ITEM_MAX_AGE,
        version != null ? version : NugetProxyFacet.V3);
  }
}
