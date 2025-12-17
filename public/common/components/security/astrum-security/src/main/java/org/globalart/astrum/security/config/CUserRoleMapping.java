
package org.globalart.astrum.security.config;

import java.io.Serializable;
import java.util.Set;

/**
 * Persistent user-role mapping.
 *
 * @since 3.0
 */
public interface CUserRoleMapping
    extends Serializable, Cloneable
{
  void addRole(final String string);

  Set<String> getRoles();

  String getSource();

  String getUserId();

  void removeRole(final String string);

  void setRoles(final Set<String> roles);

  void setSource(final String source);

  void setUserId(final String userId);

  String getVersion();

  void setVersion(final String version);

  CUserRoleMapping clone();
}
