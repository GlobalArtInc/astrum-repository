
package org.globalart.astrum.security.role;

import java.util.HashSet;
import java.util.Set;

/**
 * Role identifier encompassing source and role identifiers.
 */
public class RoleIdentifier
{
  private String source;

  private String roleId;

  public RoleIdentifier(String source, String roleId) {
    this.source = source;
    this.roleId = roleId;
  }

  public String getSource() {
    return source;
  }

  public void setSource(String source) {
    this.source = source;
  }

  public String getRoleId() {
    return roleId;
  }

  public void setRoleId(String roleId) {
    this.roleId = roleId;
  }

  public static Set<RoleIdentifier> getRoleIdentifiersForSource(String source, Set<RoleIdentifier> roleIdentifiers) {
    Set<RoleIdentifier> sourceRoleIdentifiers = new HashSet<RoleIdentifier>();

    if (roleIdentifiers != null) {
      for (RoleIdentifier roleIdentifier : roleIdentifiers) {
        if (roleIdentifier.getSource().equals(source)) {
          sourceRoleIdentifiers.add(roleIdentifier);
        }
      }
    }

    return sourceRoleIdentifiers;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }

    RoleIdentifier that = (RoleIdentifier) o;

    if (roleId != null ? !roleId.equals(that.roleId) : that.roleId != null) {
      return false;
    }
    if (source != null ? !source.equals(that.source) : that.source != null) {
      return false;
    }

    return true;
  }

  @Override
  public int hashCode() {
    int result = source != null ? source.hashCode() : 0;
    result = 31 * result + (roleId != null ? roleId.hashCode() : 0);
    return result;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "source='" + source + '\'' +
        ", roleId='" + roleId + '\'' +
        '}';
  }
}
