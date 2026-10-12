package org.globalart.astrum.repository.terraform.rest;

import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.HttpClientAttributes;
import org.globalart.astrum.repository.rest.api.model.NegativeCacheAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyAttributes;
import org.globalart.astrum.repository.rest.api.model.ReplicationAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiProxyRepository;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.repository.rest.api.model.TerraformAttributes;
import org.globalart.astrum.repository.terraform.TerraformFormat;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;

public class TerraformProxyApiRepository
    extends SimpleApiProxyRepository
{
  private final TerraformAttributes terraform;

  public TerraformProxyApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("proxy") final ProxyAttributes proxy,
      @JsonProperty("negativeCache") final NegativeCacheAttributes negativeCache,
      @JsonProperty("httpClient") final HttpClientAttributes httpClient,
      @JsonProperty("routingRuleName") final String routingRuleName,
      @JsonProperty("replication") @JsonInclude(value = Include.NON_EMPTY, content = Include.NON_NULL)
      final ReplicationAttributes replication,
      @JsonProperty("terraform") final TerraformAttributes terraform)
  {
    super(name, TerraformFormat.NAME, url, online, storage, cleanup, proxy, negativeCache, httpClient,
        routingRuleName, replication);
    this.terraform = terraform;
  }

  public TerraformAttributes getTerraform() {
    return terraform;
  }
}
