package org.globalart.astrum.repository.npm.rest;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.npm.internal.NpmFormat;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.npm.rest.NpmAttributes.NPM;
import static org.globalart.astrum.repository.npm.rest.NpmAttributes.REMOVE_NON_CATALOGED;
import static org.globalart.astrum.repository.npm.rest.NpmAttributes.REMOVE_QUARANTINED;

@Component
@Qualifier(NpmFormat.NAME)
public class NpmApiRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  @Autowired
  public NpmApiRepositoryAdapter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    if (!ProxyType.NAME.equals(repository.getType().getValue())) {
      return super.adapt(repository);
    }
    return new NpmProxyApiRepository(
        repository.getName(),
        repository.getUrl(),
        repository.getConfiguration().isOnline(),
        getHostedStorageAttributes(repository),
        getCleanupPolicyAttributes(repository),
        getProxyAttributes(repository),
        getNegativeCacheAttributes(repository),
        getHttpClientAttributes(repository),
        getRoutingRuleName(repository),
        getReplicationAttributes(repository),
        createNpmAttributes(repository));
  }

  private NpmAttributes createNpmAttributes(final Repository repository) {
    NestedAttributesMap npm = repository.getConfiguration().attributes(NPM);
    return new NpmAttributes(
        npm.get(REMOVE_QUARANTINED, Boolean.class, Boolean.FALSE),
        npm.get(REMOVE_NON_CATALOGED, Boolean.class, Boolean.FALSE));
  }
}
