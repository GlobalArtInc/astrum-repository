
package org.globalart.astrum.repository.apt.rest;

import jakarta.validation.Valid;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.repository.apt.api.AptHostedRepositoriesAttributes;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

/**
 * @since 3.20
 */
@JsonIgnoreProperties({"format", "type"})
public class AptHostedRepositoryApiRequest
    extends HostedRepositoryApiRequest
{
  @NotNull
  @Valid
  protected final AptHostedRepositoriesAttributes apt;

  @NotNull
  @Valid
  protected final AptSigningRepositoriesAttributes aptSigning;

  @JsonCreator
  public AptHostedRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("apt") final AptHostedRepositoriesAttributes apt,
      @JsonProperty("aptSigning") final AptSigningRepositoriesAttributes aptSigning,
      @JsonProperty("component") final ComponentAttributes componentAttributes)
  {
    super(name, AptFormat.NAME, online, storage, cleanup, componentAttributes);
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
