package org.globalart.astrum.oidc.internal;

import org.apache.shiro.authc.AuthenticationToken;

public class OidcAuthenticationToken
    implements AuthenticationToken
{
  private final String userId;

  private final String subject;

  public OidcAuthenticationToken(final String userId, final String subject) {
    this.userId = userId;
    this.subject = subject;
  }

  @Override
  public Object getPrincipal() {
    return userId;
  }

  @Override
  public Object getCredentials() {
    return subject;
  }
}
