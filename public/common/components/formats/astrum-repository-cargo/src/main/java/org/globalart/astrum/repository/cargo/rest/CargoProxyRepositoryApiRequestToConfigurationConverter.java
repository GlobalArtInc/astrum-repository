package org.globalart.astrum.repository.cargo.rest;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class CargoProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<CargoProxyRepositoryApiRequest>
{
  @Autowired
  public CargoProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public Configuration convert(final CargoProxyRepositoryApiRequest request) {
    return CargoRestAttributes.apply(super.convert(request), request.getCargo());
  }
}
