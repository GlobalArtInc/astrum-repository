package org.globalart.astrum.repository.terraform.rest;

import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiGroupRepository;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.repository.rest.api.model.TerraformAttributes;
import org.globalart.astrum.repository.terraform.TerraformFormat;

import com.fasterxml.jackson.annotation.JsonProperty;

public class TerraformGroupApiRepository
    extends SimpleApiGroupRepository
{
  private final TerraformAttributes terraform;

  public TerraformGroupApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupAttributes group,
      @JsonProperty("terraform") final TerraformAttributes terraform)
  {
    super(name, TerraformFormat.NAME, url, online, storage, group);
    this.terraform = terraform;
  }

  public TerraformAttributes getTerraform() {
    return terraform;
  }
}
