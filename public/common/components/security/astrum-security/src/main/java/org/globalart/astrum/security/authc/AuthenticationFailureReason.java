
package org.globalart.astrum.security.authc;

/**
 * The reason why an authentication attempt failed.
 *
 * @since 3.22
 */
public enum AuthenticationFailureReason
{
  USER_NOT_FOUND,
  PASSWORD_EMPTY,
  INCORRECT_CREDENTIALS,
  DISABLED_ACCOUNT,
  LICENSE_LIMITATION,
  EXPIRED_CREDENTIALS,
  UNKNOWN
}
