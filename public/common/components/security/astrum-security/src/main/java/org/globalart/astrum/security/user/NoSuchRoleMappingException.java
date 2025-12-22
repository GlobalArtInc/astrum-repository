
package org.globalart.astrum.security.user;

/**
 * Thrown when a role mapping cannot be found.
 */
public class NoSuchRoleMappingException
    extends Exception
{
  private static final long serialVersionUID = -8368148376838186349L;

  public NoSuchRoleMappingException(final String userId) {
    super("No user-role mapping for user: " + userId);
  }
}
