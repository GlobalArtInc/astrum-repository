package org.globalart.astrum.repository.terraform.rest;

import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedRepositoryApiRequest;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.TerraformAttributes;
import org.globalart.astrum.repository.terraform.TerraformFormat;
import org.globalart.astrum.swagger.SwaggerEditionVisibility;

import com.fasterxml.jackson.annotation.JsonCreator;
import com.fasterxml.jackson.annotation.JsonFilter;
import com.fasterxml.jackson.annotation.JsonIgnoreProperties;
import com.fasterxml.jackson.annotation.JsonProperty;

@JsonFilter(SwaggerEditionVisibility.NAME)
@JsonIgnoreProperties({"format", "type"})
public class TerraformHostedRepositoryApiRequest
    extends HostedRepositoryApiRequest
{
  private final TerraformAttributes terraform;

  private final TerraformSigningAttributes terraformSigning;

  @JsonCreator
  public TerraformHostedRepositoryApiRequest(
      @JsonProperty("name") final String name,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes componentAttributes,
      @JsonProperty("terraform") final TerraformAttributes terraform,
      @JsonProperty("terraformSigning") final TerraformSigningAttributes terraformSigning)
  {
    super(name, TerraformFormat.NAME, online, storage, cleanup, componentAttributes);
    this.terraform = terraform;
    this.terraformSigning = terraformSigning;
  }

  public TerraformAttributes getTerraform() {
    return terraform;
  }

  public TerraformSigningAttributes getTerraformSigning() {
    return terraformSigning;
  }
}
