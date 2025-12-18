
package org.globalart.astrum.security.privilege;

/**
 * @since 3.19
 */
public class PrivilegeException
    extends RuntimeException
{
  public PrivilegeException(final String message) {
    super(message);
  }

  public PrivilegeException(final String message, Exception cause) {
    super(message, cause);
  }
}
