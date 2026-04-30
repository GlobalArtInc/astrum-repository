
package org.globalart.astrum.repository.conan.internal.security.token;

import javax.annotation.Nullable;

import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenManager;

import org.apache.shiro.subject.Subject;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.security.anonymous.AnonymousHelper.isAnonymous;

@Component
public class ConanTokenManager
    extends BearerTokenManager
{
  @Autowired
  public ConanTokenManager(final ApiKeyService apiKeyService, final SecurityHelper securityHelper) {
    super(apiKeyService, securityHelper, ConanToken.NAME);
  }

  @Nullable
  public String login() {
    Subject subject = securityHelper.subject();
    if (!isAuthenticatedUser(subject)) {
      return null;
    }
    return createToken(subject.getPrincipals());
  }

  @Nullable
  public String user() {
    Subject subject = securityHelper.subject();
    if (!isAuthenticatedUser(subject)) {
      return null;
    }
    return subject.getPrincipals().getPrimaryPrincipal().toString();
  }

  private static boolean isAuthenticatedUser(@Nullable final Subject subject) {
    return subject != null && subject.getPrincipal() != null && subject.isAuthenticated() && !isAnonymous(subject);
  }
}
