package org.globalart.astrum.repository.nuget.internal.apikey;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.security.UserPrincipalsHelper;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenRealm;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(NugetApiKeyRealm.NAME)
@Description("NuGet API-Key Realm")
public class NugetApiKeyRealm
    extends BearerTokenRealm
{
  public static final String NAME = "NuGetApiKey";

  public static final String HEADER = "X-NuGet-ApiKey";

  @Autowired
  public NugetApiKeyRealm(final ApiKeyService apiKeyService, final UserPrincipalsHelper principalsHelper) {
    super(apiKeyService, principalsHelper, NAME);
  }
}
