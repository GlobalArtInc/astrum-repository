
package org.globalart.astrum.security.authc;

import java.util.Set;

import static java.util.Collections.emptySet;

/**
 * An event fired when the an user is authorized.
 *
 * @since 3.0
 */
public class AuthenticationEvent
{
  private final String userId;

  private final boolean successful;

  private final Set<AuthenticationFailureReason> authenticationFailureReasons;

  public AuthenticationEvent(final String userId, final boolean successful) {
    this(userId, successful, emptySet());
  }

  public AuthenticationEvent(
      final String userId,
      final boolean successful,
      final Set<AuthenticationFailureReason> authenticationFailureReasons)
  {
    this.userId = userId;
    this.successful = successful;
    this.authenticationFailureReasons = authenticationFailureReasons;
  }

  public String getUserId() {
    return userId;
  }

  public boolean isSuccessful() {
    return successful;
  }

  public Set<AuthenticationFailureReason> getAuthenticationFailureReasons() {
    return authenticationFailureReasons;
  }

  @Override
  public String toString() {
    return "AuthenticationEvent{" +
        "userId='" + userId + '\'' +
        ", successful=" + successful +
        ", failureReasons=" + authenticationFailureReasons +
        '}';
  }
}
