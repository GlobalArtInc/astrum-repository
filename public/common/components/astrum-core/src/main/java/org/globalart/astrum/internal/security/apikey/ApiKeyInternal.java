
package org.globalart.astrum.internal.security.apikey;

import java.time.OffsetDateTime;

import org.globalart.astrum.security.authc.apikey.ApiKey;

import org.apache.shiro.subject.PrincipalCollection;

/**
 * A database-stored object representing the association between a {@link PrincipalCollection} and a Api Key (char[]).
 */
public interface ApiKeyInternal
    extends ApiKey
{
  /**
   * Returns the domain (e.g. npm, docker, etc) which the key is applicable to.
   */
  String getDomain();

  /**
   * Set the time at which this token was created
   */
  void setCreated(OffsetDateTime created);

  /**
   * Set the domain of the token
   */
  void setDomain(String domain);

  /**
   * Sets the principals for the token, the object may not be preserved and will be constructed from the string
   * representation of primary principal and the first realm.
   */
  void setPrincipals(PrincipalCollection principals);

  /**
   * Gets the userId of the admin who created this API key.
   */
  default String getCreatedByUserId() {
    return null;
  }

  /**
   * Gets the realm of the admin who created this API key.
   */
  default String getCreatedByRealm() {
    return null;
  }
}
