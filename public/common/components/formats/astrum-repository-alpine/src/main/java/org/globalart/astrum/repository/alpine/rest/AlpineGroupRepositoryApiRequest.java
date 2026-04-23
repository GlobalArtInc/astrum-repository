package org.globalart.astrum.repository.alpine.rest;

import jakarta.validation.Valid;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class AlpineGroupRepositoryApiRequest
    extends GroupRepositoryApiRequest
{
  @Valid
  private final AlpineSigningAttributes alpineSigning;

  @JsonCreator
  public AlpineGroupRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group,
      @JsonProperty("alpineSigning") final AlpineSigningAttributes alpineSigning)
  {
    super(name, AlpineFormat.NAME, online, storage, group);
    this.alpineSigning = alpineSigning;
  }

  public AlpineSigningAttributes getAlpineSigning() {
    return alpineSigning;
  }
}
