package org.globalart.astrum.repository.cargo.rest;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.cargo.internal.CargoRepositoryConfig;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.rest.api.model.CargoAttributes;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(CargoFormat.NAME)
public class CargoApiRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  @Autowired
  public CargoApiRepositoryAdapter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    String name = repository.getName();
    String url = repository.getUrl();
    boolean online = repository.getConfiguration().isOnline();
    CargoAttributes cargo = new CargoAttributes(CargoRepositoryConfig.requireAuthentication(repository));
    return switch (repository.getType().getValue()) {
      case HostedType.NAME -> new CargoHostedApiRepository(name, url, online,
          getHostedStorageAttributes(repository),
          getCleanupPolicyAttributes(repository),
          getComponentAttributes(repository),
          cargo);
      case ProxyType.NAME -> new CargoProxyApiRepository(name, url, online,
          getStorageAttributes(repository),
          getCleanupPolicyAttributes(repository),
          getProxyAttributes(repository),
          getNegativeCacheAttributes(repository),
          getHttpClientAttributes(repository),
          getRoutingRuleName(repository),
          getReplicationAttributes(repository),
          cargo);
      case GroupType.NAME -> new CargoGroupApiRepository(name, url, online,
          getStorageAttributes(repository),
          getGroupAttributes(repository),
          cargo);
      default -> super.adapt(repository);
    };
  }
}
