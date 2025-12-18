

package org.globalart.astrum.security.internal.rest;

import java.util.HashSet;
import java.util.Set;

import javax.annotation.Nullable;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.security.role.RoleIdentifier;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserManager;

import io.swagger.v3.oas.annotations.media.Schema;
import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotBlank;

/**
 * REST API representation of a user.
 *
 * @since 3.17
 */
public class ApiUser
{
  @NotBlank
  @Schema(description = AstrumSecurityApiConstants.USER_ID_DESCRIPTION)
  private String userId;

  @Schema(description = AstrumSecurityApiConstants.FIRST_NAME_DESCRIPTION, nullable = true)
  private String firstName;

  @Schema(description = AstrumSecurityApiConstants.LAST_NAME_DESCRIPTION, nullable = true)
  private String lastName;

  @Email
  @Schema(description = AstrumSecurityApiConstants.EMAIL_DESCRIPTION, nullable = true)
  private String emailAddress;

  @NotBlank
  @Schema(description = AstrumSecurityApiConstants.SOURCE_DESCRIPTION)
  private String source;

  @NotNull
  @Schema(description = AstrumSecurityApiConstants.STATUS_DESCRIPTION)
  private ApiUserStatus status;

  @Schema(description = "Indicates whether the user's properties could be modified by the Astrum Repository Manager. "
      + "When false only roles are considered during update.")
  private boolean readOnly;

  @Schema(description = AstrumSecurityApiConstants.ROLES_DESCRIPTION)
  private Set<String> roles;

  @Schema(description = "The roles which the user has been assigned in an external source, "
      + "e.g. LDAP group. These cannot be changed within the Astrum Repository Manager.")
  private Set<String> externalRoles;

  @SuppressWarnings("unused")
  private ApiUser() {
    // deserialization
  }

  public ApiUser(
      final String userId,
      final String firstName,
      final String lastName,
      final String emailAddress,
      final String source,
      final ApiUserStatus status,
      final boolean readOnly,
      final Set<String> roles,
      final Set<String> externalRoles) // NOSONAR
  {
    this.userId = userId;
    this.firstName = firstName;
    this.lastName = lastName;
    this.emailAddress = emailAddress;
    this.source = source;
    this.status = status;
    this.readOnly = readOnly;
    this.roles = roles;
    this.externalRoles = externalRoles;
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

  public String getSource() {
    return source;
  }

  public ApiUserStatus getStatus() {
    return status;
  }

  public boolean isReadOnly() {
    return readOnly;
  }

  public Set<String> getRoles() {
    return roles;
  }

  @Nullable
  public Set<String> getExternalRoles() {
    return externalRoles;
  }

  public void setExternalRoles(final Set<String> externalRoles) {
    this.externalRoles = externalRoles;
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

  public void setSource(final String source) {
    this.source = source;
  }

  public void setStatus(final ApiUserStatus status) {
    this.status = status;
  }

  public void setReadOnly(final boolean readOnly) {
    this.readOnly = readOnly;
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
    user.setSource(source);
    user.setStatus(status.getStatus());
    user.setReadOnly(readOnly);
    user.setVersion(1);

    Set<RoleIdentifier> roleIdentifiers = new HashSet<>();
    if (roles != null) {
      roles.stream().map(r -> new RoleIdentifier(UserManager.DEFAULT_SOURCE, r)).forEach(roleIdentifiers::add);
    }
    if (externalRoles != null) {
      externalRoles.stream().map(r -> new RoleIdentifier(source, r)).forEach(roleIdentifiers::add);
    }
    user.setRoles(roleIdentifiers);
    return user;
  }
}
