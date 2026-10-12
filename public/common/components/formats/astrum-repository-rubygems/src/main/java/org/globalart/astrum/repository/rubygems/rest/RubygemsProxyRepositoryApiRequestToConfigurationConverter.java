package org.globalart.astrum.repository.rubygems.rest;

import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class RubygemsProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<RubygemsProxyRepositoryApiRequest>
{
  @Autowired
  public RubygemsProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }
}
