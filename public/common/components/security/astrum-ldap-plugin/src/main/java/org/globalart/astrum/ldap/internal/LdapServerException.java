package org.globalart.astrum.ldap.internal;

/**
 * An LDAP server could not be reached or refused an operation that should have worked.
 */
public class LdapServerException
    extends RuntimeException
{
  public LdapServerException(final String message) {
    super(message);
  }

  public LdapServerException(final String message, final Throwable cause) {
    super(message, cause);
  }
}
