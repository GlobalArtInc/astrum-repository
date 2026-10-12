package org.globalart.astrum.repository.rubygems.internal.security;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.security.UserPrincipalsHelper;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenRealm;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(RubygemsApiKey.NAME)
@Description("RubyGems API Key Realm")
public class RubygemsApiKeyRealm
    extends BearerTokenRealm
{
  @Autowired
  public RubygemsApiKeyRealm(final ApiKeyService keyStore, final UserPrincipalsHelper principalsHelper) {
    super(keyStore, principalsHelper, RubygemsApiKey.NAME);
  }
}
