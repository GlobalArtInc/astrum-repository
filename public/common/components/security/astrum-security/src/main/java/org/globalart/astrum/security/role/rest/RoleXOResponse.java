
package org.globalart.astrum.security.role.rest;

import java.util.Objects;
import java.util.Set;

import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;
import org.globalart.astrum.security.role.Role;

import io.swagger.v3.oas.annotations.media.Schema;

public class RoleXOResponse
{
  @Schema(description = AstrumSecurityApiConstants.ROLE_ID_DESCRIPTION)
  private String id;

  @Schema(description = AstrumSecurityApiConstants.ROLE_SOURCE_DESCRIPTION)
  private String source;

  @Schema(description = AstrumSecurityApiConstants.ROLE_NAME_DESCRIPTION)
  private String name;

  @Schema(description = AstrumSecurityApiConstants.ROLE_DESCRIPTION_DESCRIPTION)
  private String description;

  @Schema(description = AstrumSecurityApiConstants.ROLE_READONLY_DESCRIPTION)
  private boolean readOnly;

  @Schema(description = AstrumSecurityApiConstants.ROLE_PRIVILEGES_DESCRIPTION)
  private Set<String> privileges;

  @Schema(description = AstrumSecurityApiConstants.ROLE_ROLES_DESCRIPTION)
  private Set<String> roles;

  public void setSource(final String source) {
    this.source = source;
  }

  public void setRoles(final Set<String> roles) {
    this.roles = roles;
  }

  public void setPrivileges(final Set<String> privileges) {
    this.privileges = privileges;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public void setDescription(final String description) {
    this.description = description;
  }

  public void setReadOnly(final boolean readOnly) {
    this.readOnly = readOnly;
  }

  public String getSource() {
    return source;
  }

  public Set<String> getRoles() {
    return roles;
  }

  public Set<String> getPrivileges() {
    return privileges;
  }

  public String getName() {
    return name;
  }

  public String getId() {
    return id;
  }

  public String getDescription() {
    return description;
  }

  public Boolean getReadOnly() {
    return readOnly;
  }

  @Override
  public String toString() {
    return "id: " + id + " name: " + name + " source: " + source + " description: " + description
        + " readOnly: " + readOnly + " roles: " + roles + " privileges: " + privileges;
  }

  @Override
  public int hashCode() {
    return Objects.hash(id, name, source, description, roles, privileges);
  }

  @Override
  public boolean equals(final Object obj) {
    if (!(obj instanceof RoleXOResponse)) {
      return false;
    }

    RoleXOResponse other = (RoleXOResponse) obj;

    return Objects.equals(id, other.id) && Objects.equals(name, other.name) && Objects.equals(source, other.source)
        && Objects.equals(description, other.description) && Objects.equals(roles, other.roles) && Objects
            .equals(privileges, other.privileges);
  }

  public static RoleXOResponse fromRole(final Role input) {
    RoleXOResponse role = new RoleXOResponse();
    role.setId(input.getRoleId());
    role.setSource(input.getSource());
    role.setDescription(input.getDescription() != null ? input.getDescription() : input.getRoleId());
    role.setName(input.getName() != null ? input.getName() : input.getRoleId());
    role.setPrivileges(input.getPrivileges());
    role.setRoles(input.getRoles());
    role.setReadOnly(input.isReadOnly());

    return role;
  }
}
