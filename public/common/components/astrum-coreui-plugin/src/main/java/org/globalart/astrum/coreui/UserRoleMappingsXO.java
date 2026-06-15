
package org.globalart.astrum.coreui;

import java.util.Set;
import jakarta.validation.constraints.NotBlank;

import org.globalart.astrum.security.realm.RealmExists;
import org.globalart.astrum.security.role.RolesExist;
import org.globalart.astrum.security.user.UserExists;
import org.globalart.astrum.validation.group.Create;

/**
 * User role mappings exchange object.
 *
 * @since 3.0
 */
public class UserRoleMappingsXO
{
  @NotBlank
  @UserExists(groups = Create.class)
  private String userId;

  @NotBlank
  @RealmExists(groups = Create.class)
  private String realm;

  @RolesExist
  private Set<String> roles;

  public String getUserId() {
    return userId;
  }

  public void setUserId(String userId) {
    this.userId = userId;
  }

  public String getRealm() {
    return realm;
  }

  public void setRealm(String realm) {
    this.realm = realm;
  }

  public Set<String> getRoles() {
    return roles;
  }

  public void setRoles(Set<String> roles) {
    this.roles = roles;
  }

  @Override
  public String toString() {
    return "UserRoleMappingsXO{" +
        "userId='" + userId + '\'' +
        ", realm='" + realm + '\'' +
        ", roles=" + roles +
        '}';
  }
}
