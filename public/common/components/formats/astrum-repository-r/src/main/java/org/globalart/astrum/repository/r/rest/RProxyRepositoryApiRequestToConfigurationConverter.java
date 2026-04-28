package org.globalart.astrum.repository.r.rest;

import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class RProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<RProxyRepositoryApiRequest>
{
  @Autowired
  public RProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }
}
