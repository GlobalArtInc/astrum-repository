package org.globalart.astrum.repository.terraform.internal.security;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.security.UserPrincipalsHelper;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenRealm;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(TerraformToken.NAME)
@Description("Terraform Bearer Token Realm")
public class TerraformTokenRealm
    extends BearerTokenRealm
{
  @Autowired
  public TerraformTokenRealm(final ApiKeyService keyStore, final UserPrincipalsHelper principalsHelper) {
    super(keyStore, principalsHelper, TerraformToken.NAME);
  }
}
