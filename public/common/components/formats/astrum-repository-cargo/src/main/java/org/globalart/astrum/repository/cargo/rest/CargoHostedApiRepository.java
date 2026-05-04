package org.globalart.astrum.repository.cargo.rest;

import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.rest.api.model.CargoAttributes;
import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiHostedRepository;

import com.fasterxml.jackson.annotation.JsonProperty;

public class CargoHostedApiRepository
    extends SimpleApiHostedRepository
{
  private final CargoAttributes cargo;

  public CargoHostedApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes component,
      @JsonProperty("cargo") final CargoAttributes cargo)
  {
    super(name, CargoFormat.NAME, url, online, storage, cleanup, component);
    this.cargo = cargo;
  }

  public CargoAttributes getCargo() {
    return cargo;
  }
}
