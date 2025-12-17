
package org.globalart.astrum.security.authc;

import java.util.Set;

import org.apache.shiro.authc.AccountException;

/**
 * Exception with details about a failed authentication attempt.
 *
 * @since 3.22
 */
public class AstrumAuthenticationException
    extends AccountException
{
  private final Set<AuthenticationFailureReason> authenticationFailureReasons;

  public AstrumAuthenticationException(
      final String cause,
      final Set<AuthenticationFailureReason> authenticationFailureReason)
  {
    super(cause);
    this.authenticationFailureReasons = authenticationFailureReason;
  }

  public Set<AuthenticationFailureReason> getAuthenticationFailureReasons() {
    return authenticationFailureReasons;
  }
}
