package org.globalart.astrum.security.accesstoken;

import java.io.Serializable;

import org.globalart.astrum.security.authz.ScopedPrincipal;

import org.apache.shiro.authz.Permission;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Added to the owner's principals when a request signs in with an access token, so the token's scope applies.
 *
 * It travels with the principals into tokens issued for that request (docker, npm), so it holds plain strings only.
 */
public record AccessTokenPrincipal(String tokenId, String tokenName, String scope)
    implements ScopedPrincipal, Serializable
{
  public AccessTokenPrincipal {
    checkNotNull(tokenId);
    checkNotNull(tokenName);
    checkNotNull(scope);
  }

  @Override
  public boolean permits(final Permission permission) {
    try {
      return AccessTokenScope.fromId(scope).permits(permission);
    }
    catch (IllegalArgumentException e) {
      return false;
    }
  }

  @Override
  public String toString() {
    return "access token " + tokenName;
  }
}
