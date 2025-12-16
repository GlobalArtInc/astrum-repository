
package org.globalart.astrum.security;

import org.globalart.astrum.security.anonymous.AnonymousConfiguration;

public class TestAnonymousConfiguration
    implements AnonymousConfiguration
{
  boolean enabled;

  String realmName;

  String userId;

  @Override
  public String getRealmName() {
    return realmName;
  }

  @Override
  public String getUserId() {
    return userId;
  }

  @Override
  public boolean isEnabled() {
    return enabled;
  }

  @Override
  public void setEnabled(final boolean enabled) {
    this.enabled = enabled;
  }

  @Override
  public void setRealmName(final String realmName) {
    this.realmName = realmName;
  }

  @Override
  public void setUserId(final String userId) {
    this.userId = userId;
  }

  @Override
  public AnonymousConfiguration copy() {
    return this;
  }
}
