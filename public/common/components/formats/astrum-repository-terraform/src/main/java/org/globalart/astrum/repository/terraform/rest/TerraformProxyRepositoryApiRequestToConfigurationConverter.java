package org.globalart.astrum.repository.terraform.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class TerraformProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<TerraformProxyRepositoryApiRequest>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public TerraformProxyRepositoryApiRequestToConfigurationConverter(
      final RoutingRuleStore routingRuleStore,
      @Lazy final RepositoryManager repositoryManager)
  {
    super(routingRuleStore);
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public Configuration convert(final TerraformProxyRepositoryApiRequest request) {
    return TerraformRestAttributes.apply(super.convert(request), request.getTerraform(), repositoryManager);
  }
}
