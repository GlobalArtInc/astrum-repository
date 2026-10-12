package org.globalart.astrum.repository.gitlfs.rest;

import org.globalart.astrum.repository.gitlfs.GitLfsFormat;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class GitLfsHostedRepositoryApiRequest
    extends HostedRepositoryApiRequest
{
  @JsonCreator
  public GitLfsHostedRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes componentAttributes)
  {
    super(name, GitLfsFormat.NAME, online, storage, cleanup, componentAttributes);
  }
}
