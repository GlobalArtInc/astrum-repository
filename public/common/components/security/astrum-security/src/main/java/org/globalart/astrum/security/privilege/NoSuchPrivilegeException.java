
package org.globalart.astrum.security.privilege;

/**
 * Thrown when a {@link Privilege} could not be found.
 */
public class NoSuchPrivilegeException
    extends PrivilegeException
{
  private static final long serialVersionUID = 820651866330926246L;

  private String privilegeId;

  public NoSuchPrivilegeException(final String privilegeId) {
    super("Privilege not found: " + privilegeId);
    this.privilegeId = privilegeId;
  }

  public String getPrivilegeId() {
    return privilegeId;
  }
}
