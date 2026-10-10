package org.globalart.astrum.ldap.internal;

import com.fasterxml.jackson.annotation.JsonCreator;

/**
 * How Astrum binds to the LDAP server for searches.
 */
public enum LdapAuthScheme
{
  NONE("none"),
  SIMPLE("simple"),
  DIGEST_MD5("DIGEST-MD5"),
  CRAM_MD5("CRAM-MD5");

  private final String uiValue;

  LdapAuthScheme(final String uiValue) {
    this.uiValue = uiValue;
  }

  /**
   * The value the classic UI uses for this scheme.
   */
  public String uiValue() {
    return uiValue;
  }

  public boolean requiresCredentials() {
    return this != NONE;
  }

  public boolean usesRealm() {
    return this == DIGEST_MD5 || this == CRAM_MD5;
  }

  /**
   * Accepts {@code simple}, {@code SIMPLE}, {@code DIGEST-MD5}, {@code digest_md5} and so on; returns {@code null} for
   * a blank or unknown value.
   */
  @JsonCreator
  public static LdapAuthScheme parse(final String value) {
    return LdapEnums.parse(LdapAuthScheme.class, value);
  }
}
