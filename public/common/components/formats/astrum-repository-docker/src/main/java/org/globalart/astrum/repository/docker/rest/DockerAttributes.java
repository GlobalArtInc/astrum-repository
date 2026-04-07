package org.globalart.astrum.repository.docker.rest;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.docker.DockerConfig;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.annotation.JsonProperty;
import io.swagger.v3.oas.annotations.media.Schema;

@JsonIgnoreProperties(ignoreUnknown = true)
public class DockerAttributes
{
  @Schema(description = "Whether to allow clients to use the V1 API to interact with this repository",
      example = "false")
  private final Boolean v1Enabled;

  @Schema(description = "Whether to force authentication (Docker Bearer Token Realm required if false)",
      example = "true")
  private final Boolean forceBasicAuth;

  @Schema(description = "Create an HTTP connector at specified port", example = "8082")
  @JsonInclude(Include.NON_NULL)
  private final Integer httpPort;

  @Schema(description = "Create an HTTPS connector at specified port", example = "8083")
  @JsonInclude(Include.NON_NULL)
  private final Integer httpsPort;

  @Schema(description = "Allows to use repository name as a subdomain for docker requests", example = "docker-a")
  @JsonInclude(Include.NON_NULL)
  private final String subdomain;

  @Schema(description = "Whether the repository is reachable by path based routing at /v2/<repository>/",
      example = "true")
  @JsonInclude(Include.NON_NULL)
  private final Boolean pathEnabled;

  @JsonCreator
  public DockerAttributes(
      @JsonProperty("v1Enabled") final Boolean v1Enabled,
      @JsonProperty("forceBasicAuth") final Boolean forceBasicAuth,
      @JsonProperty("httpPort") final Integer httpPort,
      @JsonProperty("httpsPort") final Integer httpsPort,
      @JsonProperty("subdomain") final String subdomain,
      @JsonProperty("pathEnabled") final Boolean pathEnabled)
  {
    this.v1Enabled = v1Enabled;
    this.forceBasicAuth = forceBasicAuth;
    this.httpPort = httpPort;
    this.httpsPort = httpsPort;
    this.subdomain = subdomain;
    this.pathEnabled = pathEnabled;
  }

  public Boolean getV1Enabled() {
    return v1Enabled;
  }

  public Boolean getForceBasicAuth() {
    return forceBasicAuth;
  }

  public Integer getHttpPort() {
    return httpPort;
  }

  public Integer getHttpsPort() {
    return httpsPort;
  }

  public String getSubdomain() {
    return subdomain;
  }

  public Boolean getPathEnabled() {
    return pathEnabled;
  }

  public static void apply(@Nullable final DockerAttributes docker, final NestedAttributesMap target) {
    target.set(DockerConfig.V1_ENABLED, docker != null && Boolean.TRUE.equals(docker.v1Enabled));
    target.set(DockerConfig.FORCE_BASIC_AUTH, docker == null || docker.forceBasicAuth == null || docker.forceBasicAuth);
    if (docker == null) {
      return;
    }
    if (docker.httpPort != null) {
      target.set(DockerConfig.HTTP_PORT, docker.httpPort);
    }
    if (docker.httpsPort != null) {
      target.set(DockerConfig.HTTPS_PORT, docker.httpsPort);
    }
    if (docker.subdomain != null && !docker.subdomain.isBlank()) {
      target.set(DockerConfig.SUBDOMAIN, docker.subdomain.trim());
    }
    if (docker.pathEnabled != null) {
      target.set(DockerConfig.PATH_ENABLED, docker.pathEnabled);
    }
  }

  public static DockerAttributes from(final NestedAttributesMap docker) {
    return new DockerAttributes(
        DockerConfig.bool(docker.get(DockerConfig.V1_ENABLED), false),
        DockerConfig.bool(docker.get(DockerConfig.FORCE_BASIC_AUTH), false),
        DockerConfig.integer(docker.get(DockerConfig.HTTP_PORT)),
        DockerConfig.integer(docker.get(DockerConfig.HTTPS_PORT)),
        docker.get(DockerConfig.SUBDOMAIN) == null ? null : docker.get(DockerConfig.SUBDOMAIN).toString(),
        docker.get(DockerConfig.PATH_ENABLED) == null ? null
            : DockerConfig.bool(docker.get(DockerConfig.PATH_ENABLED), true));
  }
}
