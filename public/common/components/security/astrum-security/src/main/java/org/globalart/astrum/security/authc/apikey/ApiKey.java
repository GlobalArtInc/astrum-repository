
package org.globalart.astrum.security.authc.apikey;

import java.time.OffsetDateTime;

import org.apache.shiro.subject.PrincipalCollection;

/**
 * A database-stored object representing the association between a {@link PrincipalCollection} and a Api Key (char[]).
 */
public interface ApiKey
{
  String getDomain();

  char[] getApiKey();

  PrincipalCollection getPrincipals();

  OffsetDateTime getCreated();

  default String getPrimaryPrincipal() {
    if (getPrincipals() == null) {
      return null;
    }
    return getPrincipals().getPrimaryPrincipal().toString();
  }
}
