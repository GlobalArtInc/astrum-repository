
package org.globalart.astrum.internal.wonderland;

import java.util.Objects;

/**
 * @since 3.15
 */
public class UserAuthToken
{
  private final String user;

  private final String token;

  private final String realmName;

  public UserAuthToken(final String user, final String token, final String realmName) {
    this.user = user;
    this.token = token;
    this.realmName = realmName;
  }

  public String getUser() {
    return user;
  }

  public String getToken() {
    return token;
  }

  public String getRealmName() {
    return realmName;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (o == null || getClass() != o.getClass()) {
      return false;
    }
    UserAuthToken that = (UserAuthToken) o;
    return Objects.equals(user, that.user)
        && Objects.equals(token, that.token)
        && Objects.equals(realmName, that.realmName);
  }

  @Override
  public int hashCode() {
    return Objects.hash(user, token, realmName);
  }

  @Override
  public String toString() {
    return getRealmName() + ":" + getUser() + ":" + getToken();
  }
}
