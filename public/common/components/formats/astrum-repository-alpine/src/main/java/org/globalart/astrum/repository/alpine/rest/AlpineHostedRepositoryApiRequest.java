package org.globalart.astrum.repository.alpine.rest;

import jakarta.validation.Valid;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class AlpineHostedRepositoryApiRequest
    extends HostedRepositoryApiRequest
{
  @Valid
  private final AlpineSigningAttributes alpineSigning;

  @JsonCreator
  public AlpineHostedRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes componentAttributes,
      @JsonProperty("alpineSigning") final AlpineSigningAttributes alpineSigning)
  {
    super(name, AlpineFormat.NAME, online, storage, cleanup, componentAttributes);
    this.alpineSigning = alpineSigning;
  }

  public AlpineSigningAttributes getAlpineSigning() {
    return alpineSigning;
  }
}
