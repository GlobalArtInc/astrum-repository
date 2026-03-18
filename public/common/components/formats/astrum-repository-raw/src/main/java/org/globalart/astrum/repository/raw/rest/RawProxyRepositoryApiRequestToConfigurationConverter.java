
package org.globalart.astrum.repository.raw.rest;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.ContentDispositionHelper;
import org.globalart.astrum.repository.rest.api.ProxyRepositoryApiRequestToConfigurationConverter;
import org.globalart.astrum.repository.routing.RoutingRuleStore;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.RAW_QUERYPARAMS_FORWARDING_ENABLED_NAMED_VALUE;
import static org.globalart.astrum.repository.raw.rest.RawAttributes.CONTENT_DISPOSITION;
import static org.globalart.astrum.repository.raw.rest.RawAttributes.EXCLUDED_QUERY_PARAMETERS;
import static org.globalart.astrum.repository.raw.rest.RawAttributes.FORWARD_QUERY_PARAMETERS;

/**
 * @since 3.25
 */
@Component
public class RawProxyRepositoryApiRequestToConfigurationConverter
    extends ProxyRepositoryApiRequestToConfigurationConverter<RawProxyRepositoryApiRequest>
{
  private final boolean rawQueryParamsForwardingEnabled;

  private final RepositoryManager repositoryManager;

  @Autowired
  public RawProxyRepositoryApiRequestToConfigurationConverter(
      final RoutingRuleStore routingRuleStore,
      final RepositoryManager repositoryManager,
      @Value(RAW_QUERYPARAMS_FORWARDING_ENABLED_NAMED_VALUE) final boolean rawQueryParamsForwardingEnabled)
  {
    super(routingRuleStore);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.rawQueryParamsForwardingEnabled = rawQueryParamsForwardingEnabled;
  }

  @Override
  public Configuration convert(final RawProxyRepositoryApiRequest request) {
    Configuration configuration = super.convert(request);
    NestedAttributesMap configAttributes = configuration.attributes("raw");
    RawAttributes requestAttributes = request.getRaw();

    String requestedDisposition = null;
    if (requestAttributes != null && requestAttributes.getContentDisposition() != null) {
      requestedDisposition = request.getRaw().getContentDisposition().name();
    }

    // Always resolve content disposition , by default to ATTACHMENT on creation
    String contentDisposition = ContentDispositionHelper.resolveContentDisposition(
        requestedDisposition,
        repositoryManager,
        request.getName(),
        "raw");

    configAttributes.set(CONTENT_DISPOSITION, contentDisposition);

    if (requestAttributes != null && rawQueryParamsForwardingEnabled) {
      if (requestAttributes.getForwardQueryParameters() != null) {
        configAttributes.set(FORWARD_QUERY_PARAMETERS, requestAttributes.getForwardQueryParameters());
      }
      if (requestAttributes.getExcludedQueryParameters() != null) {
        configAttributes.set(EXCLUDED_QUERY_PARAMETERS, requestAttributes.getExcludedQueryParameters());
      }
    }

    return configuration;
  }
}
