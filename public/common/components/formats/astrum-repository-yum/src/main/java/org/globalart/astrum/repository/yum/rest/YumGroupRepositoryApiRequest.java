package org.globalart.astrum.repository.yum.rest;

import jakarta.validation.Valid;

import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type", "yum"})
public class YumGroupRepositoryApiRequest
    extends GroupRepositoryApiRequest
{
  @Valid
  private final YumSigningAttributes yumSigning;

  @JsonCreator
  public YumGroupRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group,
      @JsonProperty("yumSigning") final YumSigningAttributes yumSigning)
  {
    super(name, YumFormat.NAME, online, storage, group);
    this.yumSigning = yumSigning;
  }

  public YumSigningAttributes getYumSigning() {
    return yumSigning;
  }
}
