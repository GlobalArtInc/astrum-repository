package org.globalart.astrum.ldap.internal;

/**
 * Body of {@code POST /v1/security/ldap/verify-login}. The credentials are UTF-8 text encoded as base64.
 */
public class LdapVerifyLoginXO
{
  private String base64Username;

  private String base64Password;

  private LdapServerApiXO ldapServer;

  public String getBase64Username() {
    return base64Username;
  }

  public void setBase64Username(final String base64Username) {
    this.base64Username = base64Username;
  }

  public String getBase64Password() {
    return base64Password;
  }

  public void setBase64Password(final String base64Password) {
    this.base64Password = base64Password;
  }

  public LdapServerApiXO getLdapServer() {
    return ldapServer;
  }

  public void setLdapServer(final LdapServerApiXO ldapServer) {
    this.ldapServer = ldapServer;
  }
}
