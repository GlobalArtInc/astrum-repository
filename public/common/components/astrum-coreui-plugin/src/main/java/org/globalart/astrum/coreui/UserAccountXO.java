
package org.globalart.astrum.coreui;

import jakarta.validation.constraints.Email;
import jakarta.validation.constraints.NotEmpty;

/**
 * User account exchange object.
 *
 * @since 3.0
 */
public class UserAccountXO
{
  @NotEmpty
  private String userId;

  @NotEmpty
  private String firstName;

  @NotEmpty
  private String lastName;

  @Email
  @NotEmpty
  private String email;

  private Boolean external;

  public String getUserId() {
    return userId;
  }

  public void setUserId(String userId) {
    this.userId = userId;
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

  public Boolean getExternal() {
    return external;
  }

  public void setExternal(Boolean external) {
    this.external = external;
  }

  @Override
  public String toString() {
    return "UserAccountXO{" +
        "userId='" + userId + '\'' +
        ", firstName='" + firstName + '\'' +
        ", lastName='" + lastName + '\'' +
        ", email='" + email + '\'' +
        ", external=" + external +
        '}';
  }
}
