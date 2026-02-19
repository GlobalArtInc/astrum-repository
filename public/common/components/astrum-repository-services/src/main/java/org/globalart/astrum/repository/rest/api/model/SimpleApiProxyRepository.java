
package org.globalart.astrum.repository.rest.api.model;

import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.types.ProxyType;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

/**
 * API Proxy Repository for simple formats which do not have custom attributes for proxies.
 *
 * @since 3.20
 */
public class SimpleApiProxyRepository
    extends AbstractApiRepository
{
  @NotNull
  protected final StorageAttributes storage;

  protected final CleanupPolicyAttributes cleanup;

  @NotNull
  protected final ProxyAttributes proxy;

  @NotNull
  protected final NegativeCacheAttributes negativeCache;

  @NotNull
  protected final HttpClientAttributes httpClient;

  @Schema(description = "The name of the routing rule assigned to this repository")
  protected final String routingRuleName;

  protected final ReplicationAttributes replication;

  /*
   * Firewall attributes are populated cross-format by {@link ApiRepositoryAdapter#adaptDecorated}
   * after the per-format adapter constructs the proxy DTO. Keeping it as a settable property
   * (instead of a constructor parameter) means per-format subclasses don't need to thread the
   * firewall field through their own constructors. Jackson deserialization populates it via the
   * {@code @JsonProperty} setter; serialization uses the getter.
   */
  protected FirewallAttributes firewall;

  @JsonCreator(mode = JsonCreator.Mode.PROPERTIES)
  public SimpleApiProxyRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("format") final String format,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("proxy") final ProxyAttributes proxy,
      @JsonProperty("negativeCache") final NegativeCacheAttributes negativeCache,
      @JsonProperty("httpClient") final HttpClientAttributes httpClient,
      @JsonProperty("routingRuleName") final String routingRuleName,
      @JsonProperty("replication") @JsonInclude(value = Include.NON_EMPTY,
          content = Include.NON_NULL) final ReplicationAttributes replication)
  {
    super(name, format, ProxyType.NAME, url, online);
    this.storage = storage;
    this.cleanup = cleanup;
    this.proxy = proxy;
    this.negativeCache = negativeCache;
    this.httpClient = httpClient;
    this.routingRuleName = routingRuleName;
    this.replication = replication;
  }

  public StorageAttributes getStorage() {
    return storage;
  }

  public CleanupPolicyAttributes getCleanup() {
    return cleanup;
  }

  public ProxyAttributes getProxy() {
    return proxy;
  }

  public NegativeCacheAttributes getNegativeCache() {
    return negativeCache;
  }

  public HttpClientAttributes getHttpClient() {
    return httpClient;
  }

  public String getRoutingRuleName() {
    return routingRuleName;
  }

  public ReplicationAttributes getReplication() {
    return replication;
  }

  public FirewallAttributes getFirewall() {
    return firewall;
  }

  @JsonProperty("firewall")
  public void setFirewall(final FirewallAttributes firewall) {
    this.firewall = firewall;
  }
}
