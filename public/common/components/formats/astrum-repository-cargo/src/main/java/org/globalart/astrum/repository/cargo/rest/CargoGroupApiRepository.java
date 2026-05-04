package org.globalart.astrum.repository.cargo.rest;

import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.rest.api.model.CargoAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiGroupRepository;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;

import com.fasterxml.jackson.annotation.JsonProperty;

public class CargoGroupApiRepository
    extends SimpleApiGroupRepository
{
  private final CargoAttributes cargo;

  public CargoGroupApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group,
      @JsonProperty("cargo") final CargoAttributes cargo)
  {
    super(name, CargoFormat.NAME, url, online, storage, group);
    this.cargo = cargo;
  }

  public CargoAttributes getCargo() {
    return cargo;
  }
}
