package org.globalart.astrum.repository.r.rest;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class RGroupRepositoryApiRequest
    extends GroupRepositoryApiRequest
{
  @JsonCreator
  public RGroupRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group)
  {
    super(name, RFormat.NAME, online, storage, group);
  }
}
