
package org.globalart.astrum.security.privilege;

/**
 * @since 3.19
 */
public class DuplicatePrivilegeException
    extends PrivilegeException
{
  private static final String ERROR_TEXT = "Privilege %s already exists.";

  public DuplicatePrivilegeException(final String privilegeId) {
    super(String.format(ERROR_TEXT, privilegeId));
  }

  public DuplicatePrivilegeException(final String privilegeId, Exception cause) {
    super(String.format(ERROR_TEXT, privilegeId), cause);
  }
}
