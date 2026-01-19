
package org.globalart.astrum.api.extdirect.common.security.role.model;

import java.util.Set;
import jakarta.validation.constraints.NotEmpty;

import org.globalart.astrum.security.privilege.PrivilegesExist;
import org.globalart.astrum.security.role.RoleNotContainSelf;
import org.globalart.astrum.security.role.RolesExist;
import org.globalart.astrum.security.role.UniqueRoleId;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.Update;

/**
 * Role exchange object.
 *
 * @since 3.0
 */
@RoleNotContainSelf(id = "getId", roleIds = "getRoles")
public class RoleXO
{
  @NotEmpty
  @UniqueRoleId(groups = Create.class)
  private String id;

  @NotEmpty(groups = Update.class)
  private String version;

  private String source;

  @NotEmpty
  private String name;

  private String description;

  private Boolean readOnly;

  @PrivilegesExist(groups = {Create.class, Update.class})
  private Set<String> privileges;

  @RolesExist(groups = {Create.class, Update.class})
  private Set<String> roles;

  public String getId() {
    return id;
  }

  public void setId(String id) {
    this.id = id;
  }

  public String getVersion() {
    return version;
  }

  public void setVersion(String version) {
    this.version = version;
  }

  public String getSource() {
    return source;
  }

  public void setSource(String source) {
    this.source = source;
  }

  public String getName() {
    return name;
  }

  public void setName(String name) {
    this.name = name;
  }

  public String getDescription() {
    return description;
  }

  public void setDescription(String description) {
    this.description = description;
  }

  public Boolean getReadOnly() {
    return readOnly;
  }

  public void setReadOnly(Boolean readOnly) {
    this.readOnly = readOnly;
  }

  public Set<String> getPrivileges() {
    return privileges;
  }

  public void setPrivileges(Set<String> privileges) {
    this.privileges = privileges;
  }

  public Set<String> getRoles() {
    return roles;
  }

  public void setRoles(Set<String> roles) {
    this.roles = roles;
  }

  @Override
  public String toString() {
    return "RoleXO{" +
        "id='" + id + '\'' +
        ", version='" + version + '\'' +
        ", source='" + source + '\'' +
        ", name='" + name + '\'' +
        ", description='" + description + '\'' +
        ", readOnly=" + readOnly +
        ", privileges=" + privileges +
        ", roles=" + roles +
        '}';
  }
}
