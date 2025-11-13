
package org.globalart.astrum.internal.security.model;

import org.globalart.astrum.common.entity.HasStringId;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.security.config.CUser;

import com.fasterxml.jackson.annotation.JsonIgnore;

/**
 * {@link CUser} data.
 *
 * @since 3.21
 */
public class CUserData
    implements HasStringId, CUser
{
  private String email;

  private String firstName;

  private String id;

  private String lastName;

  private String password;

  private String status;

  private int version = 1;

  @Override
  public String getEmail() {
    return this.email;
  }

  @Override
  public String getFirstName() {
    return this.firstName;
  }

  @Override
  public String getId() {
    return this.id;
  }

  @Override
  public String getLastName() {
    return this.lastName;
  }

  @Override
  public String getPassword() {
    return this.password;
  }

  @Override
  public String getStatus() {
    return this.status;
  }

  @Override
  public int getVersion() {
    return version;
  }

  @JsonIgnore
  @Override
  public boolean isActive() {
    return STATUS_ACTIVE.equals(status) || STATUS_CHANGE_PASSWORD.equals(status);
  }

  @Override
  public void setEmail(final String email) {
    this.email = email;
  }

  @Override
  public void setFirstName(final String firstName) {
    this.firstName = firstName;
  }

  @Override
  public void setId(final String id) {
    this.id = id;
  }

  @Override
  public void setLastName(final String lastName) {
    this.lastName = lastName;
  }

  @Override
  public void setPassword(final String password) {
    this.password = password;
  }

  @Override
  public void setStatus(final String status) {
    this.status = status;
  }

  @Override
  public void setVersion(final int version) {
    this.version = Math.max(version, 1);
  }

  @Override
  public CUserData clone() {
    try {
      return (CUserData) super.clone();
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "id='" + id + '\'' +
        ", firstName='" + firstName + '\'' +
        ", lastName='" + lastName + '\'' +
        ", password='" + Strings2.mask(password) + '\'' +
        ", status='" + status + '\'' +
        ", email='" + email + '\'' +
        ", version='" + version + '\'' +
        '}';
  }
}
