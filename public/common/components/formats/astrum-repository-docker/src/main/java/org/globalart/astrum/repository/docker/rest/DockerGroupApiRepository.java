package org.globalart.astrum.repository.docker.rest;

import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.rest.api.model.GroupDeployAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiGroupDeployRepository;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

public class DockerGroupApiRepository
    extends SimpleApiGroupDeployRepository
{
  private final DockerAttributes docker;

  @JsonCreator
  public DockerGroupApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupDeployAttributes group,
      @JsonProperty("docker") final DockerAttributes docker)
  {
    super(name, DockerFormat.NAME, url, online, storage, group);
    this.docker = docker;
  }

  public DockerAttributes getDocker() {
    return docker;
  }
}
