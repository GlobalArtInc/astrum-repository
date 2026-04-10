
package org.globalart.astrum.repository.apt.api;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * Adapter to expose Apt specific properties for the repositories REST API.
 *
 * @since 3.20
 */
@Component
@Qualifier(AptFormat.NAME)
public class AptApiRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  @Autowired
  public AptApiRepositoryAdapter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    boolean online = repository.getConfiguration().isOnline();
    String name = repository.getName();
    String url = repository.getUrl();

    switch (repository.getType().toString()) {
      case HostedType.NAME:
        return new AptHostedApiRepository(
            name,
            url,
            online,
            getHostedStorageAttributes(repository),
            getCleanupPolicyAttributes(repository),
            createAptHostedRepositoriesAttributes(repository),
            createAptSigningRepositoriesAttributes(repository),
            getComponentAttributes(repository));
      case ProxyType.NAME:
        return new AptProxyApiRepository(name, url, online,
            getStorageAttributes(repository),
            getCleanupPolicyAttributes(repository),
            createAptProxyRepositoriesAttributes(repository),
            createAptSigningRepositoriesAttributes(repository),
            getProxyAttributes(repository),
            getNegativeCacheAttributes(repository),
            getHttpClientAttributes(repository),
            getRoutingRuleName(repository),
            getReplicationAttributes(repository));
    }
    return null;
  }

  private AptHostedRepositoriesAttributes createAptHostedRepositoriesAttributes(final Repository repository) {
    String distribution = repository.getConfiguration().attributes(AptFormat.NAME).get("distribution", String.class);
    return new AptHostedRepositoriesAttributes(distribution);
  }

  private AptSigningRepositoriesAttributes createAptSigningRepositoriesAttributes(final Repository repository) {
    NestedAttributesMap aptAttributes = repository.getConfiguration().attributes("aptSigning");
    String keypair = aptAttributes.get("keypair", String.class);
    if (!Strings2.isBlank(keypair)) {
      return new AptSigningRepositoriesAttributes(keypair, null);
    }
    return null;
  }

  private AptProxyRepositoriesAttributes createAptProxyRepositoriesAttributes(final Repository repository) {
    NestedAttributesMap aptAttributes = repository.getConfiguration().attributes(AptFormat.NAME);
    String distribution = aptAttributes.get("distribution", String.class);
    Boolean flat = aptAttributes.get("flat", Boolean.class);
    Boolean enforceDistribution = aptAttributes.get("enforceDistribution", Boolean.class);
    return new AptProxyRepositoriesAttributes(distribution, flat, enforceDistribution);
  }
}
