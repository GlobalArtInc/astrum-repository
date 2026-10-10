package org.globalart.astrum.ldap.internal;

import com.fasterxml.jackson.annotation.JsonCreator;

public enum LdapProtocol
{
  LDAP(389),
  LDAPS(636);

  private final int defaultPort;

  LdapProtocol(final int defaultPort) {
    this.defaultPort = defaultPort;
  }

  public int defaultPort() {
    return defaultPort;
  }

  public String scheme() {
    return name().toLowerCase();
  }

  /**
   * Accepts the name in any case; returns {@code null} for a blank or unknown value.
   */
  @JsonCreator
  public static LdapProtocol parse(final String value) {
    return LdapEnums.parse(LdapProtocol.class, value);
  }
}
