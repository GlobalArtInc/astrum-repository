package org.globalart.astrum.repository.rest.api.model;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

public class TerraformAttributes
{
  public static final String REQUIRE_AUTHENTICATION = "requireAuthentication";

  public static final String REGISTRY_HOST = "registryHost";

  public static final String UPSTREAM_HOST = "upstreamHost";

  @Schema(description = "Indicates if this repository requires authentication overriding anonymous access.",
      example = "false")
  protected Boolean requireAuthentication;

  @Schema(description = "Host name, with an optional port, that Terraform configurations use as the registry host "
      + "of this repository. Requests for /.well-known/terraform.json on that host are answered with the endpoints "
      + "of this repository.", example = "terraform.example.com")
  protected String registryHost;

  @Schema(description = "Proxy repositories only: the host name provider addresses of the upstream registry carry, "
      + "which the provider network mirror protocol names. Defaults to the host of the remote URL; set it when the "
      + "remote is another mirror of that registry.", example = "registry.terraform.io")
  protected String upstreamHost;

  public TerraformAttributes() {
    this.requireAuthentication = null;
  }

  public TerraformAttributes(final Boolean requireAuthentication) {
    this(requireAuthentication, null, null);
  }

  @JsonCreator
  public TerraformAttributes(
      @JsonProperty(REQUIRE_AUTHENTICATION) final Boolean requireAuthentication,
      @JsonProperty(REGISTRY_HOST) final String registryHost,
      @JsonProperty(UPSTREAM_HOST) final String upstreamHost)
  {
    this.requireAuthentication = requireAuthentication;
    this.registryHost = registryHost;
    this.upstreamHost = upstreamHost;
  }

  public Boolean getRequireAuthentication() {
    return requireAuthentication;
  }

  public void setRequireAuthentication(final Boolean requireAuthentication) {
    this.requireAuthentication = requireAuthentication;
  }

  public String getRegistryHost() {
    return registryHost;
  }

  public void setRegistryHost(final String registryHost) {
    this.registryHost = registryHost;
  }

  public String getUpstreamHost() {
    return upstreamHost;
  }

  public void setUpstreamHost(final String upstreamHost) {
    this.upstreamHost = upstreamHost;
  }
}
