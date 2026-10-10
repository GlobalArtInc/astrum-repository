package org.globalart.astrum.ldap.internal;

public class LdapServerNotFoundException
    extends RuntimeException
{
  public LdapServerNotFoundException(final String server) {
    super("LDAP server '" + server + "' does not exist");
  }
}
