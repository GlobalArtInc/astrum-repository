package org.globalart.astrum.repository.cargo.rest;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;
import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.rest.api.model.CargoAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupDeployAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class CargoGroupRepositoryApiRequest
    extends GroupRepositoryApiRequest
{
  private final CargoAttributes cargo;

  @JsonCreator
  public CargoGroupRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupDeployAttributes group,
      @JsonProperty("cargo") final CargoAttributes cargo)
  {
    super(name, CargoFormat.NAME, online, storage, group);
    this.cargo = cargo;
  }

  public CargoAttributes getCargo() {
    return cargo;
  }
}
