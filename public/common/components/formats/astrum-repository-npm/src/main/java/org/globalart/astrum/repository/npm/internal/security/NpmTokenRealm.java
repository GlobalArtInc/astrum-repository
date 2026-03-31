
package org.globalart.astrum.repository.npm.internal.security;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.npm.security.NpmToken;

import org.globalart.astrum.security.UserPrincipalsHelper;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenRealm;

import org.apache.shiro.realm.AuthenticatingRealm;
import org.apache.shiro.subject.Subject;
import org.globalart.astrum.common.Description;

/**
 * {@link AuthenticatingRealm} that maps npm tokens to valid {@link Subject}s.
 *
 * @since 3.0
 */
@Component
@Qualifier(NpmToken.NAME)
@Description("npm Bearer Token Realm")
public class NpmTokenRealm
    extends BearerTokenRealm
{
  @Autowired
  public NpmTokenRealm(final ApiKeyService keyStore, final UserPrincipalsHelper principalsHelper) {
    super(keyStore, principalsHelper, NpmToken.NAME);
  }
}
