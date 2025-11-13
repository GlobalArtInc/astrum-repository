
package org.globalart.astrum.internal.security.model;

import java.util.Set;

import org.globalart.astrum.common.entity.HasStringId;
import org.globalart.astrum.security.config.CRole;

import com.google.common.collect.Sets;

/**
 * {@link CRole} data.
 *
 * @since 3.21
 */
public class CRoleData
    implements HasStringId, CRole
{
  private String description;

  private String id;

  private String name;

  private Set<String> privileges;

  private boolean readOnly = false;

  private Set<String> roles;

  private int version = 1;

  @Override
  public void addPrivilege(final String string) {
    getPrivileges().add(string);
  }

  @Override
  public void addRole(final String string) {
    getRoles().add(string);
  }

  @Override
  public String getDescription() {
    return this.description;
  }

  @Override
  public String getId() {
    return this.id;
  }

  @Override
  public String getName() {
    return this.name;
  }

  @Override
  public Set<String> getPrivileges() {
    if (this.privileges == null) {
      this.privileges = Sets.newHashSet();
    }

    return this.privileges;
  }

  @Override
  public Set<String> getRoles() {
    if (this.roles == null) {
      this.roles = Sets.newHashSet();
    }

    return this.roles;
  }

  @Override
  public int getVersion() {
    return version;
  }

  @Override
  public boolean isReadOnly() {
    return this.readOnly;
  }

  @Override
  public void removePrivilege(final String string) {
    getPrivileges().remove(string);
  }

  @Override
  public void removeRole(final String string) {
    getRoles().remove(string);
  }

  @Override
  public void setDescription(final String description) {
    this.description = description;
  }

  @Override
  public void setId(final String id) {
    this.id = id;
  }

  @Override
  public void setName(final String name) {
    this.name = name;
  }

  @Override
  public void setPrivileges(final Set<String> privileges) {
    this.privileges = privileges;
  }

  @Override
  public void setReadOnly(final boolean readOnly) {
    this.readOnly = readOnly;
  }

  @Override
  public void setRoles(final Set<String> roles) {
    this.roles = roles;
  }

  @Override
  public void setVersion(final int version) {
    this.version = Math.max(version, 1);
  }

  @Override
  public CRoleData clone() {
    try {
      CRoleData copy = (CRoleData) super.clone();

      if (this.privileges != null) {
        copy.privileges = Sets.newHashSet(this.privileges);
      }

      if (this.roles != null) {
        copy.roles = Sets.newHashSet(this.roles);
      }

      return copy;
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "id='" + id + '\'' +
        ", name='" + name + '\'' +
        ", description='" + description + '\'' +
        ", privileges=" + privileges +
        ", roles=" + roles +
        ", readOnly=" + readOnly +
        ", version='" + version + '\'' +
        '}';
  }
}
