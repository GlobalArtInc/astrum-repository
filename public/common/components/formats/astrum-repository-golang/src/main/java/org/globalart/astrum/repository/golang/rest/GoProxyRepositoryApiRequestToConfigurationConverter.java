package org.globalart.astrum.repository.golang.rest;

import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class GoProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<GoProxyRepositoryApiRequest>
{
  @Autowired
  public GoProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }
}
