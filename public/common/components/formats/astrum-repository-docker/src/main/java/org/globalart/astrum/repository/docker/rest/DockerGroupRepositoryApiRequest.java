package org.globalart.astrum.repository.docker.rest;

import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.rest.api.model.GroupDeployAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupDeployRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import jakarta.validation.Valid;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class DockerGroupRepositoryApiRequest
    extends GroupDeployRepositoryApiRequest
{
  @Valid
  private final DockerAttributes docker;

  @JsonCreator
  public DockerGroupRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupDeployAttributes group,
      @JsonProperty("docker") final DockerAttributes docker)
  {
    super(name, DockerFormat.NAME, online, storage, group);
    this.docker = docker;
  }

  public DockerAttributes getDocker() {
    return docker;
  }
}
