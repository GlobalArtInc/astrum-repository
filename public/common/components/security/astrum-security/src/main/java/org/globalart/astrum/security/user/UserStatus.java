
package org.globalart.astrum.security.user;

/**
 * Enum of possible User statuses.
 */
public enum UserStatus
{
  active(true),
  locked(false),
  disabled(false),
  changepassword(true);

  private boolean isActive;

  UserStatus(boolean isActive) {
    this.isActive = isActive;
  }

  public boolean isActive() {
    return isActive;
  }
}
