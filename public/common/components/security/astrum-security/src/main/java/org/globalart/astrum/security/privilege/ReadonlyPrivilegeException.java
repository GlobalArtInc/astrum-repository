
package org.globalart.astrum.security.privilege;

/**
 * @since 3.19
 */
public class ReadonlyPrivilegeException
    extends PrivilegeException
{
  private static final String ERROR_TEXT = "Privilege %s is read only.";

  public ReadonlyPrivilegeException(final String privilegeId) {
    super(String.format(ERROR_TEXT, privilegeId));
  }

  public ReadonlyPrivilegeException(final String privilegeId, Exception cause) {
    super(String.format(ERROR_TEXT, privilegeId), cause);
  }
}
