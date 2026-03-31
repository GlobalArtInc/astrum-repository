
package org.globalart.astrum.repository.npm.internal.security;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;


import org.globalart.astrum.repository.npm.security.NpmToken;

import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.security.authc.apikey.ApiKeyService;
import org.globalart.astrum.security.token.BearerTokenManager;

import org.apache.shiro.authc.AuthenticationException;
import org.apache.shiro.authc.AuthenticationInfo;
import org.apache.shiro.authc.UsernamePasswordToken;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * npm api key manager.
 *
 * @since 3.0
 */
@Component
public class NpmTokenManager
    extends BearerTokenManager
{
  @Autowired
  public NpmTokenManager(final ApiKeyService apiKeyService, final SecurityHelper securityHelper) {
    super(apiKeyService, securityHelper, NpmToken.NAME);
  }

  /**
   * Verifies passed in principal/credentials combo, and creates (if not already exists) a npm token mapped to given
   * principal and returns the newly created token.
   */
  public String login(final String username, final String password) {
    checkNotNull(username);
    checkNotNull(password);

    try {
      AuthenticationInfo authenticationInfo = securityHelper.getSecurityManager().authenticate(
          new UsernamePasswordToken(username, password));
      return super.createToken(authenticationInfo.getPrincipals());
    }
    catch (AuthenticationException e) {
      log.debug("Bad credentials provided for npm token creation", e);
      return null;
    }
  }

  /**
   * Removes any npm API Key token for current user, if exists, and returns {@code true}.
   */
  public boolean logout() {
    return super.deleteToken();
  }
}
