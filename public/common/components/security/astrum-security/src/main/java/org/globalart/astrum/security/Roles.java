
package org.globalart.astrum.security;

/**
 * Holder for Astrum role related constants.
 *
 * @since 3.0
 */
public final class Roles
{
  /**
   * Role ID used for the Administrator role, used in some places to detect is user admin or not.
   */
  public static final String ADMIN_ROLE_ID = "as-admin";

  /**
   * Role ID used for the Anonymous role, used in some places to detect is used anonymous or not.
   */
  public static final String ANONYMOUS_ROLE_ID = "as-anonymous";

  private Roles() {
    // nop
  }
}
