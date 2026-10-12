package org.globalart.astrum.repository.yum.rest;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.yum.YumFormat;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(YumFormat.NAME)
public class YumApiRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  @Autowired
  public YumApiRepositoryAdapter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    String name = repository.getName();
    String url = repository.getUrl();
    boolean online = repository.getConfiguration().isOnline();
    return switch (repository.getType().getValue()) {
      case HostedType.NAME -> new YumHostedApiRepository(name, url, online,
          getHostedStorageAttributes(repository),
          getCleanupPolicyAttributes(repository),
          getComponentAttributes(repository),
          YumAttributes.of(repository),
          YumSigningAttributes.of(repository));
      case ProxyType.NAME -> new YumProxyApiRepository(name, url, online,
          getStorageAttributes(repository),
          getCleanupPolicyAttributes(repository),
          getProxyAttributes(repository),
          getNegativeCacheAttributes(repository),
          getHttpClientAttributes(repository),
          getRoutingRuleName(repository),
          getReplicationAttributes(repository),
          YumSigningAttributes.of(repository));
      case GroupType.NAME -> new YumGroupApiRepository(name, url, online,
          getStorageAttributes(repository),
          getGroupAttributes(repository),
          YumSigningAttributes.of(repository));
      default -> super.adapt(repository);
    };
  }
}
