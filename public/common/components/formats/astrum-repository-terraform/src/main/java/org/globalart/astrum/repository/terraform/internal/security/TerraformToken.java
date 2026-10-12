package org.globalart.astrum.repository.terraform.internal.security;

import org.globalart.astrum.security.token.BearerToken;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

/**
 * Reads the {@code Authorization: Bearer TerraformToken.<key>} header Terraform sends for the credentials of a
 * host, given in a {@code credentials} block of the CLI configuration or a {@code TF_TOKEN_<host>} variable.
 */
@Component
@Qualifier(TerraformToken.NAME)
public class TerraformToken
    extends BearerToken
{
  public static final String NAME = "TerraformToken";

  public TerraformToken() {
    super(NAME);
  }
}
