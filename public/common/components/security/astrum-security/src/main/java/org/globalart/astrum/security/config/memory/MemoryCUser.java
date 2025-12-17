
package org.globalart.astrum.security.config.memory;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.security.config.CUser;

/**
 * An implementation of {@link CUser} suitable for an in-memory backing store.
 *
 * @since 3.0
 */
public class MemoryCUser
    implements CUser
{
  private String email;

  private String firstName;

  private String id;

  private String lastName;

  private String password;

  private String status;

  private int version;

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
    return password;
  }

  @Override
  public String getStatus() {
    return this.status;
  }

  @Override
  public int getVersion() {
    return version;
  }

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
    this.version = version;
  }

  public MemoryCUser withEmail(final String email) {
    setEmail(email);
    return this;
  }

  public MemoryCUser withFirstName(final String firstName) {
    setFirstName(firstName);
    return this;
  }

  public MemoryCUser withId(final String id) {
    setId(id);
    return this;
  }

  public MemoryCUser withLastName(final String lastName) {
    setLastName(lastName);
    return this;
  }

  public MemoryCUser withPassword(final String password) {
    setPassword(password);
    return this;
  }

  public MemoryCUser withStatus(final String status) {
    setStatus(status);
    return this;
  }

  public MemoryCUser withVersion(final int version) {
    setVersion(version);
    return this;
  }

  @Override
  public MemoryCUser clone() {
    try {
      return (MemoryCUser) super.clone();
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
