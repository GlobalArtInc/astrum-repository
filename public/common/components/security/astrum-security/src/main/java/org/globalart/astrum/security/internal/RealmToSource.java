
package org.globalart.astrum.security.internal;

import java.util.Arrays;

/**
 * This is meant to be a utility enum that manages the mapping of realms against user
 * sources (An authenticating realm has a name that matches a user source).
 * 
 * @author Alejandro Tovar
 */
public enum RealmToSource
{
  SAML_USER_SOURCE("SamlRealm", "SAML"),
  OAUTH2_USER_SOURCE("OAuth2Realm", "OAuth2"),
  CROWD_USER_SOURCE("Crowd", "Crowd"),
  LDAP_USER_SOURCE("LdapRealm", "LDAP"),
  ASTRUM_USER_SOURCE("AstrumAuthenticatingRealm", "default");

  private final String realmName;

  private final String sourceName;

  RealmToSource(final String realmName, final String sourceName) {
    this.realmName = realmName;
    this.sourceName = sourceName;
  }

  /**
   * Method that searches for a user source based on a realm name
   * 
   * @param realm to be queried for a valid source for a user
   * @return String value that represent the user source for a realm
   */
  public static String getSource(String realm) {
    return Arrays.stream(values())
        .filter(keyRealm -> keyRealm.realmName.equals(realm))
        .map(keyRealm -> keyRealm.sourceName)
        .findFirst()
        .orElse("default");
  }
}
