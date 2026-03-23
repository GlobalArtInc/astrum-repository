
package org.globalart.astrum.repository.maven.api;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiHostedRepository;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * REST API model for a maven hosted repository.
 *
 * @since 3.20
 */
public class MavenHostedApiRepository
    extends SimpleApiHostedRepository
{
  @Valid
  @NotNull
  protected final MavenAttributes maven;

  @JsonCreator
  public MavenHostedApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("maven") final MavenAttributes maven,
      @JsonProperty("component") final ComponentAttributes component)
  {
    super(name, Maven2Format.NAME, url, online, storage, cleanup, component);
    this.maven = maven;
  }

  public MavenAttributes getMaven() {
    return maven;
  }
}
