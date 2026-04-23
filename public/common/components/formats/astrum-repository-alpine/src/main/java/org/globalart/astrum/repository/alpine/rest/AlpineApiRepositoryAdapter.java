package org.globalart.astrum.repository.alpine.rest;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(AlpineFormat.NAME)
public class AlpineApiRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  @Autowired
  public AlpineApiRepositoryAdapter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    String name = repository.getName();
    String url = repository.getUrl();
    boolean online = repository.getConfiguration().isOnline();
    return switch (repository.getType().getValue()) {
      case HostedType.NAME -> new AlpineHostedApiRepository(name, url, online,
          getHostedStorageAttributes(repository),
          getCleanupPolicyAttributes(repository),
          getComponentAttributes(repository),
          AlpineSigningAttributes.of(repository));
      case ProxyType.NAME -> new AlpineProxyApiRepository(name, url, online,
          getStorageAttributes(repository),
          getCleanupPolicyAttributes(repository),
          getProxyAttributes(repository),
          getNegativeCacheAttributes(repository),
          getHttpClientAttributes(repository),
          getRoutingRuleName(repository),
          getReplicationAttributes(repository),
          AlpineSigningAttributes.of(repository));
      case GroupType.NAME -> new AlpineGroupApiRepository(name, url, online,
          getStorageAttributes(repository),
          getGroupAttributes(repository),
          AlpineSigningAttributes.of(repository));
      default -> super.adapt(repository);
    };
  }
}
