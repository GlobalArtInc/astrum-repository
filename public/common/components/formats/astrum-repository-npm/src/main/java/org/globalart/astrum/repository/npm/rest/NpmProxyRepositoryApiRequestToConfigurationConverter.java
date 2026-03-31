package org.globalart.astrum.repository.npm.rest;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.npm.rest.NpmAttributes.NPM;
import static org.globalart.astrum.repository.npm.rest.NpmAttributes.REMOVE_NON_CATALOGED;
import static org.globalart.astrum.repository.npm.rest.NpmAttributes.REMOVE_QUARANTINED;

@Component
public class NpmProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<NpmProxyRepositoryApiRequest>
{
  @Autowired
  public NpmProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public Configuration convert(final NpmProxyRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    NpmAttributes npm = request.getNpm();
    if (npm != null) {
      NestedAttributesMap attributes = configuration.attributes(NPM);
      if (npm.getRemoveQuarantined() != null) {
        attributes.set(REMOVE_QUARANTINED, npm.getRemoveQuarantined());
      }
      if (npm.getRemoveNonCataloged() != null) {
        attributes.set(REMOVE_NON_CATALOGED, npm.getRemoveNonCataloged());
      }
    }
    return configuration;
  }
}
