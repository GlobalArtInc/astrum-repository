
package org.globalart.astrum.repository.maven.rest;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import com.fasterxml.jackson.annotation.JsonAlias;
import org.globalart.astrum.repository.maven.api.MavenAttributes;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.NegativeCacheAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyRepositoryApiRequest;
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
public class MavenProxyRepositoryApiRequest
    extends ProxyRepositoryApiRequest
{
  @Valid
  @NotNull
  protected final MavenAttributes maven;

  @Valid
  @NotNull
  protected final HttpClientAttributesWithPreemptiveAuth httpClient;

  @JsonCreator(mode = JsonCreator.Mode.PROPERTIES)
  @SuppressWarnings("squid:S00107") // suppress constructor parameter count
  public MavenProxyRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("proxy") final ProxyAttributes proxy,
      @JsonProperty("negativeCache") final NegativeCacheAttributes negativeCache,
      @JsonProperty("httpClient") final HttpClientAttributesWithPreemptiveAuth httpClient,
      @JsonProperty("routingRuleName") @JsonAlias("routingRule") final String routingRule,
      @JsonProperty("maven") final MavenAttributes maven,
      @JsonProperty("replication") @JsonInclude(value = Include.NON_EMPTY,
          content = Include.NON_NULL) final ReplicationAttributes replication)
  {
    super(name, Maven2Format.NAME, online, storage, cleanup, proxy, negativeCache, httpClient, routingRule,
        replication);
    this.maven = maven;
    this.httpClient = httpClient;
  }

  public MavenAttributes getMaven() {
    return maven;
  }

  @Override
  public HttpClientAttributesWithPreemptiveAuth getHttpClient() {
    return httpClient;
  }
}
