package org.globalart.astrum.repository.terraform.rest;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(TerraformFormat.NAME)
public class TerraformApiRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  @Autowired
  public TerraformApiRepositoryAdapter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    String name = repository.getName();
    String url = repository.getUrl();
    boolean online = repository.getConfiguration().isOnline();
    return switch (repository.getType().getValue()) {
      case HostedType.NAME -> new TerraformHostedApiRepository(name, url, online,
          getHostedStorageAttributes(repository),
          getCleanupPolicyAttributes(repository),
          getComponentAttributes(repository),
          TerraformRestAttributes.of(repository),
          TerraformSigningAttributes.of(repository));
      case ProxyType.NAME -> new TerraformProxyApiRepository(name, url, online,
          getStorageAttributes(repository),
          getCleanupPolicyAttributes(repository),
          getProxyAttributes(repository),
          getNegativeCacheAttributes(repository),
          getHttpClientAttributes(repository),
          getRoutingRuleName(repository),
          getReplicationAttributes(repository),
          TerraformRestAttributes.of(repository));
      case GroupType.NAME -> new TerraformGroupApiRepository(name, url, online,
          getStorageAttributes(repository),
          getGroupAttributes(repository),
          TerraformRestAttributes.of(repository));
      default -> super.adapt(repository);
    };
  }
}
