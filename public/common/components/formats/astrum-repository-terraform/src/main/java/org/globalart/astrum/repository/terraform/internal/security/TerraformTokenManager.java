package org.globalart.astrum.repository.terraform.internal.security;

import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenManager;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class TerraformTokenManager
    extends BearerTokenManager
{
  @Autowired
  public TerraformTokenManager(final ApiKeyService apiKeyService, final SecurityHelper securityHelper) {
    super(apiKeyService, securityHelper, TerraformToken.NAME);
  }

  public String getTokenForCurrentUser() {
    return createToken(securityHelper.subject().getPrincipals());
  }
}
