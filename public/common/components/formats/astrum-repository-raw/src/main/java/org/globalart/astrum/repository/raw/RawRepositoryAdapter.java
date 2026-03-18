
package org.globalart.astrum.repository.raw;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.rest.api.SimpleApiRepositoryAdapter;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.FeatureFlags.RAW_QUERYPARAMS_FORWARDING_ENABLED_NAMED_VALUE;

/**
 * Adapter to expose raw specific repository configuration for the repositories REST API.
 *
 * @since 3.41
 */
@Component
@Qualifier(RawFormat.NAME)
public class RawRepositoryAdapter
    extends SimpleApiRepositoryAdapter
{
  private static final String RAW = "raw";

  private final boolean rawQueryParamsForwardingEnabled;

  @Autowired
  public RawRepositoryAdapter(
      final RoutingRuleStore routingRuleStore,
      @Value(RAW_QUERYPARAMS_FORWARDING_ENABLED_NAMED_VALUE) final boolean rawQueryParamsForwardingEnabled)
  {
    super(routingRuleStore);
    this.rawQueryParamsForwardingEnabled = rawQueryParamsForwardingEnabled;
  }

  @Override
  public AbstractApiRepository adapt(final Repository repository) {
    return switch (repository.getType().toString()) {
      case HostedType.NAME -> new RawHostedApiRepository(
          repository.getName(),
          repository.getUrl(),
          repository.getConfiguration().isOnline(),
          getHostedStorageAttributes(repository),
          getCleanupPolicyAttributes(repository),
          getComponentAttributes(repository),
          createRawAttributes(repository));
      case ProxyType.NAME -> new RawProxyApiRepository(
          repository.getName(),
          repository.getUrl(),
          repository.getConfiguration().isOnline(),
          getHostedStorageAttributes(repository),
          getCleanupPolicyAttributes(repository),
          getProxyAttributes(repository),
          getNegativeCacheAttributes(repository),
          getHttpClientAttributes(repository),
          getRoutingRuleName(repository),
          getReplicationAttributes(repository),
          createRawAttributes(repository));
      case GroupType.NAME -> new RawGroupApiRepository(
          repository.getName(),
          repository.getUrl(),
          repository.getConfiguration().isOnline(),
          getHostedStorageAttributes(repository),
          getGroupAttributes(repository),
          createRawAttributes(repository));
      default -> throw new IllegalArgumentException("Unsupported repository type: " + repository.getType());
    };
  }

  @SuppressWarnings("unchecked")
  private RawAttributes createRawAttributes(final Repository repository) {
    NestedAttributesMap rawAttributes = repository.getConfiguration().attributes(RAW);

    // Normalize null to INLINE - legacy repos with null serve inline at handler level
    String disposition = Optional.ofNullable(rawAttributes.get("contentDisposition", String.class))
        .orElse(ContentDisposition.INLINE.name());

    if (!rawQueryParamsForwardingEnabled) {
      return new RawAttributes(disposition);
    }
    Boolean forwardQueryParameters = rawAttributes.get("forwardQueryParameters", Boolean.class);
    List<String> excludedQueryParameters =
        rawAttributes.get("excludedQueryParameters", List.class, Collections.emptyList());
    return new RawAttributes(disposition, forwardQueryParameters, excludedQueryParameters);
  }
}
