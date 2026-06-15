
package org.globalart.astrum.coreui;

import java.util.Set;
import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.security.role.RolesExist;
import org.globalart.astrum.security.user.UniqueUserId;
import org.globalart.astrum.security.user.UserStatus;
import org.globalart.astrum.validation.group.Create;
import org.globalart.astrum.validation.group.Update;

/**
 * User exchange object.
 *
 * @since 3.0
 */
public class UserXO
{
  @NotBlank
  @UniqueUserId(groups = Create.class)
  private String userId;

  @NotBlank(groups = Update.class)
  private String version;

  // Null on create
  private String realm;

  @NotBlank
  private String firstName;

  @NotBlank
  private String lastName;

  @NotBlank
  @Email
  private String email;

  @NotNull
  private UserStatus status;

  @NotBlank(groups = Create.class)
  private String password;

  @NotEmpty
  @RolesExist(groups = {Create.class, Update.class})
  private Set<String> roles;

  private Boolean external;

  // FIXME: Sort out what this is used for
  private Set<String> externalRoles;

  public String getUserId() {
    return userId;
  }

  public void setUserId(String userId) {
    this.userId = userId;
  }

  public String getVersion() {
    return version;
  }

  public void setVersion(String version) {
    this.version = version;
  }

  public String getRealm() {
    return realm;
  }

  public void setRealm(String realm) {
    this.realm = realm;
  }

  public String getFirstName() {
    return firstName;
  }

  public void setFirstName(String firstName) {
    this.firstName = firstName;
  }

  public String getLastName() {
    return lastName;
  }

  public void setLastName(String lastName) {
    this.lastName = lastName;
  }

  public String getEmail() {
    return email;
  }

  public void setEmail(String email) {
    this.email = email;
  }

  public UserStatus getStatus() {
    return status;
  }

  public void setStatus(UserStatus status) {
    this.status = status;
  }

  public String getPassword() {
    return password;
  }

  public void setPassword(String password) {
    this.password = password;
  }

  public Set<String> getRoles() {
    return roles;
  }

  public void setRoles(Set<String> roles) {
    this.roles = roles;
  }

  public Boolean isExternal() {
    return external;
  }

  public void setExternal(Boolean external) {
    this.external = external;
  }

  public Set<String> getExternalRoles() {
    return externalRoles;
  }

  public void setExternalRoles(Set<String> externalRoles) {
    this.externalRoles = externalRoles;
  }

  @Override
  public String toString() {
    return "UserXO{" +
        "userId='" + userId + '\'' +
        ", version='" + version + '\'' +
        ", realm='" + realm + '\'' +
        ", firstName='" + firstName + '\'' +
        ", lastName='" + lastName + '\'' +
        ", email='" + email + '\'' +
        ", status=" + status +
        ", password='" + password + '\'' +
        ", roles=" + roles +
        ", external=" + external +
        ", externalRoles=" + externalRoles +
        '}';
  }
}
