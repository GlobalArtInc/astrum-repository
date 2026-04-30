package org.globalart.astrum.repository.conan.rest;

import java.util.Locale;

import org.globalart.astrum.repository.conan.internal.proxy.ConanProxyFacet;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class ConanProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<ConanProxyRepositoryApiRequest>
{
  @Autowired
  public ConanProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public Configuration convert(final ConanProxyRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    ConanProxyAttributes attributes = request.getConanProxy();
    String version = attributes != null ? attributes.getConanVersion() : null;
    String normalized = version == null || version.isBlank()
        ? ConanProxyFacet.V2
        : version.trim().toUpperCase(Locale.ROOT);
    if (!ConanProxyFacet.V1.equals(normalized) && !ConanProxyFacet.V2.equals(normalized)) {
      throw new ValidationErrorsException("conanProxy.conanVersion", "must be V1 or V2");
    }
    configuration.attributes(ConanProxyFacet.CONFIG_KEY).set(ConanProxyFacet.CONAN_VERSION, normalized);
    return configuration;
  }
}
