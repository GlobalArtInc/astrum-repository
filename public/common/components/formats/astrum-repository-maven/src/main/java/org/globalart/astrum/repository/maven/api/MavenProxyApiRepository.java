
package org.globalart.astrum.repository.maven.api;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.maven.rest.HttpClientAttributesWithPreemptiveAuth;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.NegativeCacheAttributes;
import org.globalart.astrum.repository.rest.api.model.ProxyAttributes;
import org.globalart.astrum.repository.rest.api.model.ReplicationAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiProxyRepository;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * REST API model for a maven proxy repository.
 *
 * @since 3.20
 */
public class MavenProxyApiRepository
    extends SimpleApiProxyRepository
{
  @Valid
  @NotNull
  protected final MavenAttributes maven;

  @SuppressWarnings("squid:S00107") // suppress constructor parameter count
  @JsonCreator
  public MavenProxyApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("proxy") final ProxyAttributes proxy,
      @JsonProperty("negativeCache") final NegativeCacheAttributes negativeCache,
      @JsonProperty("httpClient") final HttpClientAttributesWithPreemptiveAuth httpClient,
      @JsonProperty("routingRuleName") final String routingRuleName,
      @JsonProperty("maven") final MavenAttributes maven,
      @JsonProperty("replication") @JsonInclude(value = Include.NON_EMPTY,
          content = Include.NON_NULL) final ReplicationAttributes replication)

  {
    super(name, Maven2Format.NAME, url, online, storage, cleanup, proxy, negativeCache, httpClient, routingRuleName,
        replication);
    this.maven = maven;
  }

  public MavenAttributes getMaven() {
    return maven;
  }
}
