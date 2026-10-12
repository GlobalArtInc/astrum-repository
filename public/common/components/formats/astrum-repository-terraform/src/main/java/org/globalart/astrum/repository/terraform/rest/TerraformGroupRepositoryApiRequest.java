package org.globalart.astrum.repository.terraform.rest;

import org.globalart.astrum.repository.rest.api.model.GroupDeployAttributes;
import org.globalart.astrum.repository.rest.api.model.GroupRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;
import org.globalart.astrum.repository.rest.api.model.TerraformAttributes;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class TerraformGroupRepositoryApiRequest
    extends GroupRepositoryApiRequest
{
  private final TerraformAttributes terraform;

  @JsonCreator
  public TerraformGroupRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final StorageAttributes storage,
      @JsonProperty("group") final GroupDeployAttributes group,
      @JsonProperty("terraform") final TerraformAttributes terraform)
  {
    super(name, TerraformFormat.NAME, online, storage, group);
    this.terraform = terraform;
  }

  public TerraformAttributes getTerraform() {
    return terraform;
  }
}
