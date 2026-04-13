
package org.globalart.astrum.repository.apt.rest;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import com.fasterxml.jackson.annotation.JsonAlias;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.apt.api.AptSigningRepositoriesAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.HttpClientAttributes;
import org.globalart.astrum.repository.rest.api.model.NegativeCacheAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyAttributes;
import org.globalart.astrum.repository.rest.api.model.ReplicationAttributes;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * @since 3.20
 */
@JsonIgnoreProperties({"format", "type"})
public class AptProxyRepositoryApiRequest
    extends ProxyRepositoryApiRequest
{
  @NotNull
  @Valid
  protected final AptProxyRepositoriesAttributes apt;

  @Valid
  protected final AptSigningRepositoriesAttributes aptSigning;

  @SuppressWarnings("squid:S00107") // suppress constructor parameter count
  @JsonCreator(mode = JsonCreator.Mode.PROPERTIES)
  public AptProxyRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("apt") final AptProxyRepositoriesAttributes apt,
      @JsonProperty("aptSigning") final AptSigningRepositoriesAttributes aptSigning,
      @JsonProperty("proxy") final ProxyAttributes proxy,
      @JsonProperty("negativeCache") final NegativeCacheAttributes negativeCache,
      @JsonProperty("httpClient") final HttpClientAttributes httpClient,
      @JsonProperty("routingRuleName") @JsonAlias("routingRule") final String routingRule,
      @JsonProperty("replication") @JsonInclude(value = Include.NON_EMPTY,
          content = Include.NON_NULL) final ReplicationAttributes replication)
  {
    super(name, AptFormat.NAME, online, storage, cleanup, proxy, negativeCache, httpClient, routingRule, replication);
    this.apt = apt;
    this.aptSigning = aptSigning;
  }

  public AptProxyRepositoriesAttributes getApt() {
    return apt;
  }

  public AptSigningRepositoriesAttributes getAptSigning() {
    return aptSigning;
  }
}
