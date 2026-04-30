package org.globalart.astrum.repository.conan.rest;

import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.HttpClientAttributes;
import org.globalart.astrum.repository.rest.api.model.NegativeCacheAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyAttributes;
import org.globalart.astrum.repository.rest.api.model.ReplicationAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiProxyRepository;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;

public class ConanProxyApiRepository
    extends SimpleApiProxyRepository
{
  private final ConanProxyAttributes conanProxy;

  public ConanProxyApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("proxy") final ProxyAttributes proxy,
      @JsonProperty("negativeCache") final NegativeCacheAttributes negativeCache,
      @JsonProperty("httpClient") final HttpClientAttributes httpClient,
      @JsonProperty("routingRuleName") final String routingRuleName,
      @JsonProperty("replication") @JsonInclude(value = Include.NON_EMPTY,
          content = Include.NON_NULL) final ReplicationAttributes replication,
      @JsonProperty("conanProxy") final ConanProxyAttributes conanProxy)
  {
    super(name, ConanFormat.NAME, url, online, storage, cleanup, proxy, negativeCache, httpClient, routingRuleName,
        replication);
    this.conanProxy = conanProxy;
  }

  public ConanProxyAttributes getConanProxy() {
    return conanProxy;
  }
}
