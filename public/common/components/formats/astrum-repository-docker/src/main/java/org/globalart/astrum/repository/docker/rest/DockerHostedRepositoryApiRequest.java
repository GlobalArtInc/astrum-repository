package org.globalart.astrum.repository.docker.rest;

import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.DockerHostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedRepositoryApiRequest;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import jakarta.validation.Valid;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class DockerHostedRepositoryApiRequest
    extends HostedRepositoryApiRequest
{
  @Valid
  private final DockerAttributes docker;

  @JsonCreator
  public DockerHostedRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final DockerHostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes componentAttributes,
      @JsonProperty("docker") final DockerAttributes docker)
  {
    super(name, DockerFormat.NAME, online, storage, cleanup, componentAttributes);
    this.docker = docker;
  }

  public DockerAttributes getDocker() {
    return docker;
  }
}
