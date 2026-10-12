package org.globalart.astrum.repository.terraform.rest;

import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.HttpClientAttributes;
import org.globalart.astrum.repository.rest.api.model.NegativeCacheAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.ReplicationAttributes;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.repository.rest.api.model.TerraformAttributes;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonAlias;
import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class TerraformProxyRepositoryApiRequest
    extends ProxyRepositoryApiRequest
{
  private final TerraformAttributes terraform;

  @JsonCreator
  public TerraformProxyRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("proxy") final ProxyAttributes proxy,
      @JsonProperty("negativeCache") final NegativeCacheAttributes negativeCache,
      @JsonProperty("httpClient") final HttpClientAttributes httpClient,
      @JsonProperty("routingRuleName") @JsonAlias("routingRule") final String routingRule,
      @JsonProperty("replication") @JsonInclude(value = Include.NON_EMPTY, content = Include.NON_NULL)
      final ReplicationAttributes replication,
      @JsonProperty("terraform") final TerraformAttributes terraform)
  {
    super(name, TerraformFormat.NAME, online, storage, cleanup, proxy, negativeCache, httpClient, routingRule,
        replication);
    this.terraform = terraform;
  }

  public TerraformAttributes getTerraform() {
    return terraform;
  }
}
