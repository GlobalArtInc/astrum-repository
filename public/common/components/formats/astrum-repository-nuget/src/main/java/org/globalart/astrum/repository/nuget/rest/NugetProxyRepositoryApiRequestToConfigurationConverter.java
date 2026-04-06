package org.globalart.astrum.repository.nuget.rest;

import java.util.Locale;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.nuget.internal.proxy.NugetProxyFacet;
import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.nuget.rest.NugetAttributes.DEFAULT_QUERY_CACHE_ITEM_MAX_AGE;
import static org.globalart.astrum.repository.nuget.rest.NugetAttributes.NUGET_VERSION;
import static org.globalart.astrum.repository.nuget.rest.NugetAttributes.QUERY_CACHE_ITEM_MAX_AGE;

@Component
public class NugetProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<NugetProxyRepositoryApiRequest>
{
  @Autowired
  public NugetProxyRepositoryApiRequestToConfigurationConverter(final RoutingRuleStore routingRuleStore) {
    super(routingRuleStore);
  }

  @Override
  public Configuration convert(final NugetProxyRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    NugetAttributes attributes = request.getNugetProxy();
    Integer maxAge = attributes != null ? attributes.getQueryCacheItemMaxAge() : null;
    String version = attributes != null ? attributes.getNugetVersion() : null;
    if (maxAge != null && maxAge < 0) {
      throw new ValidationErrorsException("nugetProxy.queryCacheItemMaxAge", "must be greater than or equal to 0");
    }
    String normalizedVersion = version == null || version.isBlank()
        ? NugetProxyFacet.V3
        : version.trim().toUpperCase(Locale.ROOT);
    if (!NugetProxyFacet.V2.equals(normalizedVersion) && !NugetProxyFacet.V3.equals(normalizedVersion)) {
      throw new ValidationErrorsException("nugetProxy.nugetVersion", "must be V2 or V3");
    }
    NestedAttributesMap nugetProxy = configuration.attributes(NugetProxyFacet.CONFIG_KEY);
    nugetProxy.set(QUERY_CACHE_ITEM_MAX_AGE, maxAge != null ? maxAge : DEFAULT_QUERY_CACHE_ITEM_MAX_AGE);
    nugetProxy.set(NUGET_VERSION, normalizedVersion);
    return configuration;
  }
}
