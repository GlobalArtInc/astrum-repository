
package org.globalart.astrum.security.anonymous;

import org.globalart.astrum.security.internal.AuthorizingRealmImpl;

/**
 * Anonymous configuration.
 *
 * @since 3.0
 */
public interface AnonymousConfiguration // NOSONAR
    extends Cloneable
{
  /**
   * @since 3.1
   */
  String DEFAULT_USER_ID = "anonymous";

  /**
   * @since 3.1
   */
  String DEFAULT_REALM_NAME = AuthorizingRealmImpl.NAME;

  /**
   * Obtain a copy of this configuration.
   */
  AnonymousConfiguration copy();

  /**
   * Get the realm in which the UserID associated with the configuration is located.
   */
  String getRealmName();

  /**
   * Get the UserID which is used as the template for permissions.
   */
  String getUserId();

  /**
   * Indicates whether anonymous access is enabled in this configuration.
   *
   * @return
   */
  boolean isEnabled();

  /**
   * Set whether anonymous access is enabled in this configuration.
   */
  void setEnabled(final boolean enabled);

  /**
   * Set the realm in which the UserID associated with the configuration is located.
   */
  void setRealmName(final String realmName);

  /**
   * Set the UserID which is used as the template for permissions.
   */
  void setUserId(final String userId);
}
