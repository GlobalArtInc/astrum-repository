package org.globalart.astrum.repository.yum.rest;

import jakarta.validation.Valid;

import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class YumHostedRepositoryApiRequest
    extends HostedRepositoryApiRequest
{
  @Valid
  private final YumAttributes yum;

  @Valid
  private final YumSigningAttributes yumSigning;

  @JsonCreator
  public YumHostedRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes componentAttributes,
      @JsonProperty("yum") final YumAttributes yum,
      @JsonProperty("yumSigning") final YumSigningAttributes yumSigning)
  {
    super(name, YumFormat.NAME, online, storage, cleanup, componentAttributes);
    this.yum = yum;
    this.yumSigning = yumSigning;
  }

  public YumAttributes getYum() {
    return yum;
  }

  public YumSigningAttributes getYumSigning() {
    return yumSigning;
  }
}
