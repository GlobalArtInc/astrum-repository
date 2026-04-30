package org.globalart.astrum.repository.conan.rest;

import com.fasterxml.jackson.annotation.JsonAlias;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;
import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.HttpClientAttributes;
import org.globalart.astrum.repository.rest.api.model.NegativeCacheAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.ReplicationAttributes;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class ConanProxyRepositoryApiRequest
    extends ProxyRepositoryApiRequest
{
  private final ConanProxyAttributes conanProxy;

  @JsonCreator
  public ConanProxyRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("proxy") final ProxyAttributes proxy,
      @JsonProperty("conanProxy") final ConanProxyAttributes conanProxy,
      @JsonProperty("negativeCache") final NegativeCacheAttributes negativeCache,
      @JsonProperty("httpClient") final HttpClientAttributes httpClient,
      @JsonProperty("routingRuleName") @JsonAlias("routingRule") final String routingRule,
      @JsonProperty("replication") @JsonInclude(value = Include.NON_EMPTY, content = Include.NON_NULL)
      final ReplicationAttributes replication)
  {
    super(name, ConanFormat.NAME, online, storage, cleanup, proxy, negativeCache, httpClient, routingRule, replication);
    this.conanProxy = conanProxy;
  }

  public ConanProxyAttributes getConanProxy() {
    return conanProxy;
  }
}
