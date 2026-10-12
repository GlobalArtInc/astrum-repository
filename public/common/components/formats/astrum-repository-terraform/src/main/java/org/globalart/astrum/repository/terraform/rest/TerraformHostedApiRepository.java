package org.globalart.astrum.repository.terraform.rest;

import org.globalart.astrum.repository.rest.api.model.CleanupPolicyAttributes;
import org.globalart.astrum.repository.rest.api.model.ComponentAttributes;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.rest.api.model.SimpleApiHostedRepository;
import org.globalart.astrum.repository.rest.api.model.TerraformAttributes;
import org.globalart.astrum.repository.terraform.TerraformFormat;

import com.fasterxml.jackson.annotation.JsonInclude;
import com.fasterxml.jackson.annotation.JsonProperty;

public class TerraformHostedApiRepository
    extends SimpleApiHostedRepository
{
  private final TerraformAttributes terraform;

  @JsonInclude(JsonInclude.Include.NON_NULL)
  private final TerraformSigningAttributes terraformSigning;

  public TerraformHostedApiRepository(
      @JsonProperty("name") final String name,
      @JsonProperty("url") final String url,
      @JsonProperty("online") final Boolean online,
      @JsonProperty("storage") final HostedStorageAttributes storage,
      @JsonProperty("cleanup") final CleanupPolicyAttributes cleanup,
      @JsonProperty("component") final ComponentAttributes component,
      @JsonProperty("terraform") final TerraformAttributes terraform,
      @JsonProperty("terraformSigning") final TerraformSigningAttributes terraformSigning)
  {
    super(name, TerraformFormat.NAME, url, online, storage, cleanup, component);
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
