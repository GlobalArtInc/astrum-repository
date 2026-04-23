package org.globalart.astrum.repository.alpine.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
public class AlpineProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<AlpineProxyRepositoryApiRequest>
{
  private final RepositoryManager repositoryManager;

  @Autowired
  public AlpineProxyRepositoryApiRequestToConfigurationConverter(
      final RoutingRuleStore routingRuleStore,
      @Lazy final RepositoryManager repositoryManager)
  {
    super(routingRuleStore);
    this.repositoryManager = checkNotNull(repositoryManager);
  }

  @Override
  public Configuration convert(final AlpineProxyRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    AlpineSigningAttributes.apply(configuration, request.getAlpineSigning(), repositoryManager.get(request.getName()));
    return configuration;
  }
}
