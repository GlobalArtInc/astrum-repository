
package org.globalart.astrum.security.role.rest;

import java.util.Objects;
import java.util.Set;

import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.NotEmpty;

public class RoleXORequest
{
  @NotEmpty
  @Schema(description = AstrumSecurityApiConstants.ROLE_ID_DESCRIPTION)
  private String id;

  @NotEmpty
  @Schema(description = AstrumSecurityApiConstants.ROLE_NAME_DESCRIPTION)
  private String name;

  @Schema(description = AstrumSecurityApiConstants.ROLE_DESCRIPTION_DESCRIPTION)
  private String description;

  @Schema(description = AstrumSecurityApiConstants.ROLE_PRIVILEGES_DESCRIPTION)
  private Set<String> privileges;

  @Schema(description = AstrumSecurityApiConstants.ROLE_ROLES_DESCRIPTION)
  private Set<String> roles;

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

  @Override
  public String toString() {
    return "id: " + id + " name: " + name + " description: " + description + " roles: " + roles
        + " privileges: " + privileges;
  }

  @Override
  public int hashCode() {
    return Objects.hash(id, name, description, roles, privileges);
  }

  @Override
  public boolean equals(final Object obj) {
    if (!(obj instanceof RoleXORequest other)) {
      return false;
    }

    return Objects.equals(id, other.id) && Objects.equals(name, other.name) && Objects
        .equals(description, other.description) && Objects.equals(roles, other.roles)
        && Objects
            .equals(privileges, other.privileges);
  }
}
