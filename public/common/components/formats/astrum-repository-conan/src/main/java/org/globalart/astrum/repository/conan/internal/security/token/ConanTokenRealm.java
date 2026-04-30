
package org.globalart.astrum.repository.conan.internal.security.token;

import org.globalart.astrum.common.Description;
import org.globalart.astrum.security.UserPrincipalsHelper;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenRealm;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

@Component
@Qualifier(ConanToken.NAME)
@Description("Conan Bearer Token Realm")
public class ConanTokenRealm
    extends BearerTokenRealm
{
  @Autowired
  public ConanTokenRealm(final ApiKeyService keyStore, final UserPrincipalsHelper principalsHelper) {
    super(keyStore, principalsHelper, ConanToken.NAME);
  }
}
