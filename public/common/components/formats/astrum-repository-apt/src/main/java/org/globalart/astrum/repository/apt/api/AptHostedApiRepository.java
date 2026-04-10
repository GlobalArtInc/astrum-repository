
package org.globalart.astrum.repository.apt.api;

import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiHostedRepository;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * REST API model representing an Apt repository.
 * 
 * @since 3.20
 */
public class AptHostedApiRepository
    extends SimpleApiHostedRepository
{
  @NotNull
  protected final AptHostedRepositoriesAttributes apt;

  @NotNull
  protected final AptSigningRepositoriesAttributes aptSigning;

  @JsonCreator
  public AptHostedApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("apt") final AptHostedRepositoriesAttributes apt,
      @JsonProperty("aptSigning") final AptSigningRepositoriesAttributes aptSigning,
      @JsonProperty("component") final ComponentAttributes component)
  {
    super(name, AptFormat.NAME, url, online, storage, cleanup, component);
    this.apt = apt;
    this.aptSigning = aptSigning;
  }

  public AptHostedRepositoriesAttributes getApt() {
    return apt;
  }

  public AptSigningRepositoriesAttributes getAptSigning() {
    return aptSigning;
  }
}
