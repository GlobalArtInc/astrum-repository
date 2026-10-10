package org.globalart.astrum.ldap.internal;

import com.fasterxml.jackson.annotation.JsonCreator;

/**
 * Where group membership comes from: a group entry that lists its members (static), or an attribute on the user entry
 * that names the user's groups (dynamic).
 */
public enum LdapGroupType
{
  STATIC,
  DYNAMIC;

  public String uiValue() {
    return name().toLowerCase();
  }

  @JsonCreator
  public static LdapGroupType parse(final String value) {
    return LdapEnums.parse(LdapGroupType.class, value);
  }
}
