

package org.globalart.astrum.api.rest.selfhosted.user.model;

import java.util.Collections;
import java.util.Set;
import java.util.stream.Collectors;

import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.security.internal.rest.ApiUserStatus;
import org.globalart.astrum.security.internal.rest.AstrumSecurityApiConstants;
import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserManager;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;
import jakarta.validation.constraints.NotEmpty;

/**
 * @since 3.17
 */
public class ApiCreateUser
{
  @NotBlank
  @Schema(description = AstrumSecurityApiConstants.USER_ID_DESCRIPTION)
  private String userId;

  @NotEmpty
  @Schema(description = AstrumSecurityApiConstants.FIRST_NAME_DESCRIPTION)
  private String firstName;

  @NotEmpty
  @Schema(description = AstrumSecurityApiConstants.LAST_NAME_DESCRIPTION)
  private String lastName;

  @Email
  @NotEmpty
  @Schema(description = AstrumSecurityApiConstants.EMAIL_DESCRIPTION)
  private String emailAddress;

  @NotEmpty
  @Schema(description = "The password for the new user.")
  private String password;

  @NotNull
  @Schema(description = AstrumSecurityApiConstants.STATUS_DESCRIPTION)
  private ApiUserStatus status;

  @Schema(description = AstrumSecurityApiConstants.ROLES_DESCRIPTION)
  private Set<String> roles;

  @SuppressWarnings("unused")
  private ApiCreateUser() {
    // for deserialization
  }

  public ApiCreateUser(
      final String userId,
      final String firstName,
      final String lastName,
      final String emailAddress,
      final String password,
      final ApiUserStatus status,
      final Set<String> roles)
  {
    this.userId = userId;
    this.firstName = firstName;
    this.lastName = lastName;
    this.emailAddress = emailAddress;
    this.password = password;
    this.status = status;
    this.roles = roles;
  }

  public String getUserId() {
    return userId;
  }

  public String getFirstName() {
    return firstName;
  }

  public String getLastName() {
    return lastName;
  }

  public String getEmailAddress() {
    return emailAddress;
  }

  public ApiUserStatus getStatus() {
    return status;
  }

  public String getPassword() {
    return password;
  }

  public Set<String> getRoles() {
    return roles;
  }

  public void setUserId(final String userId) {
    this.userId = userId;
  }

  public void setFirstName(final String firstName) {
    this.firstName = firstName;
  }

  public void setLastName(final String lastName) {
    this.lastName = lastName;
  }

  public void setEmailAddress(final String emailAddress) {
    this.emailAddress = emailAddress;
  }

  public void setPassword(final String password) {
    this.password = password;
  }

  public void setStatus(final ApiUserStatus status) {
    this.status = status;
  }

  public void setRoles(final Set<String> roles) {
    this.roles = roles;
  }

  public User toUser() {
    User user = new User();
    user.setUserId(userId);
    user.setFirstName(firstName);
    user.setLastName(lastName);
    user.setEmailAddress(emailAddress);
    user.setStatus(status.getStatus());
    user.setReadOnly(false);
    user.setVersion(1);
    user.setSource(UserManager.DEFAULT_SOURCE);
    if (roles != null) {
      user.setRoles(roles.stream()
          .map(r -> new RoleIdentifier(UserManager.DEFAULT_SOURCE, r))
          .collect(Collectors.toSet()));
    }
    else {
      user.setRoles(Collections.emptySet());
    }
    return user;
  }
}
