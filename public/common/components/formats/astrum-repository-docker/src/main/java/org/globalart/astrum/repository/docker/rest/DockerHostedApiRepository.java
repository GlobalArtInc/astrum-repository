package org.globalart.astrum.repository.docker.rest;

import org.globalart.astrum.repository.docker.DockerFormat;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.DockerHostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiHostedRepository;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

public class DockerHostedApiRepository
    extends SimpleApiHostedRepository
{
  private final DockerAttributes docker;

  @JsonCreator
  public DockerHostedApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final DockerHostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes component,
      @JsonProperty("docker") final DockerAttributes docker)
  {
    super(name, DockerFormat.NAME, url, online, storage, cleanup, component);
    this.docker = docker;
  }

  public DockerAttributes getDocker() {
    return docker;
  }
}
