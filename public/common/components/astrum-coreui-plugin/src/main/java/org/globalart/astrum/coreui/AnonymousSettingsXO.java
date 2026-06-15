
package org.globalart.astrum.coreui;

/**
 * Anonymous Security Settings exchange object.
 */
public class AnonymousSettingsXO
{
  private Boolean enabled;

  private String userId;

  private String realmName;

  public Boolean getEnabled() {
    return enabled;
  }

  public void setEnabled(final Boolean enabled) {
    this.enabled = enabled;
  }

  public String getUserId() {
    return userId;
  }

  public void setUserId(final String userId) {
    this.userId = userId;
  }

  public String getRealmName() {
    return realmName;
  }

  public void setRealmName(final String realmName) {
    this.realmName = realmName;
  }

  @Override
  public String toString() {
    return "AnonymousSettingsXO(" +
        "enabled:" + enabled +
        ", userId:" + userId +
        ", realmName:" + realmName +
        ")";
  }
}
