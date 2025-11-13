
package org.globalart.astrum.internal.security.anonymous;

import java.util.Objects;

import org.globalart.astrum.security.anonymous.AnonymousConfiguration;

/**
 * {@link AnonymousConfiguration} data.
 *
 * @since 3.21
 */
public class AnonymousConfigurationData
    implements AnonymousConfiguration, Cloneable
{
  private boolean enabled;

  private String userId;

  private String realmName;

  @Override
  public boolean isEnabled() {
    return enabled;
  }

  @Override
  public void setEnabled(final boolean enabled) {
    this.enabled = enabled;
  }

  @Override
  public String getUserId() {
    return userId;
  }

  @Override
  public void setUserId(final String userId) {
    this.userId = userId;
  }

  @Override
  public String getRealmName() {
    return realmName;
  }

  @Override
  public void setRealmName(final String realmName) {
    this.realmName = realmName;
  }

  @Override
  public AnonymousConfiguration copy() {
    try {
      return (AnonymousConfiguration) clone();
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    AnonymousConfigurationData that = (AnonymousConfigurationData) o;
    return enabled == that.enabled &&
        Objects.equals(userId, that.userId) &&
        Objects.equals(realmName, that.realmName);
  }

  @Override
  public int hashCode() {
    return Objects.hash(enabled, userId, realmName);
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "enabled=" + enabled +
        ", userId='" + userId + '\'' +
        ", realmName='" + realmName + '\'' +
        '}';
  }
}
