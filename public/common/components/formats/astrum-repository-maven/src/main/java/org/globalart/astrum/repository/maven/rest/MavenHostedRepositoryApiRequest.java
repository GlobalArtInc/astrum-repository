
package org.globalart.astrum.repository.maven.rest;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.maven.api.MavenAttributes;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * @since 3.20
 */
@JsonIgnoreProperties({"format", "type"})
public class MavenHostedRepositoryApiRequest
    extends HostedRepositoryApiRequest
{
  @Valid
  @NotNull
  protected final MavenAttributes maven;

  @JsonCreator
  public MavenHostedRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("maven") final MavenAttributes maven,
      @JsonProperty("component") final ComponentAttributes componentAttributes)
  {
    super(name, Maven2Format.NAME, online, storage, cleanup, componentAttributes);
    this.maven = maven;
  }

  public MavenAttributes getMaven() {
    return maven;
  }
}
