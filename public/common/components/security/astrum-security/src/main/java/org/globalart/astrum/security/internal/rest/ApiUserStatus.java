

package org.globalart.astrum.security.internal.rest;

import org.globalart.astrum.security.user.UserStatus;

/**
 * @since 3.17
 */
public enum ApiUserStatus
{
  active(UserStatus.active),
  locked(UserStatus.locked),
  disabled(UserStatus.disabled),
  changepassword(
      UserStatus.changepassword); // NOSONAR

  private UserStatus status;

  ApiUserStatus(final UserStatus status) {
    this.status = status;
  }

  public UserStatus getStatus() {
    return status;
  }

  public static ApiUserStatus convert(final UserStatus status) {
    if (status == null) {
      return null;
    }
    return ApiUserStatus.valueOf(status.toString());
  }
}
