package org.globalart.astrum.repository.rubygems.internal.security;

import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenManager;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

@Component
public class RubygemsApiKeyManager
    extends BearerTokenManager
{
  @Autowired
  public RubygemsApiKeyManager(final ApiKeyService apiKeyService, final SecurityHelper securityHelper) {
    super(apiKeyService, securityHelper, RubygemsApiKey.NAME);
  }

  public String getKeyForCurrentUser() {
    return createToken(securityHelper.subject().getPrincipals());
  }
}
