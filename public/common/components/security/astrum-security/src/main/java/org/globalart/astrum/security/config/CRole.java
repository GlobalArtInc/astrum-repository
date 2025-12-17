
package org.globalart.astrum.security.config;

import java.io.Serializable;
import java.util.Set;

/**
 * Persistent role.
 *
 * @since 3.0
 */
public interface CRole
    extends Cloneable, Serializable
{
  void addPrivilege(String string);

  void addRole(String string);

  String getDescription();

  String getId();

  String getName();

  Set<String> getPrivileges();

  Set<String> getRoles();

  boolean isReadOnly();

  void removePrivilege(String string);

  void removeRole(String string);

  void setDescription(String description);

  void setId(String id);

  void setName(String name);

  void setPrivileges(final Set<String> privileges);

  void setReadOnly(boolean readOnly);

  void setRoles(Set<String> roles);

  int getVersion();

  void setVersion(final int version);

  CRole clone();
}
