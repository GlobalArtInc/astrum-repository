package org.globalart.astrum.repository.huggingface.rest;

import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class HuggingFaceProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<HuggingFaceProxyRepositoryApiRequest>
{
  @Autowired
  public HuggingFaceProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }
}
