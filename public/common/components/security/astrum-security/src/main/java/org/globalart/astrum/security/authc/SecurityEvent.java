
package org.globalart.astrum.security.authc;

public abstract class SecurityEvent
{
  private final String realm;

  private final String principal;

  protected SecurityEvent(final String principal, final String realm) {
    this.realm = realm;
    this.principal = principal;
  }

  public String getRealm() {
    return realm;
  }

  public String getPrincipal() {
    return principal;
  }
}
