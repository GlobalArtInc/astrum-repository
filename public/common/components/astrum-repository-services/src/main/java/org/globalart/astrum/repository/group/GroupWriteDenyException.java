
package org.globalart.astrum.repository.group;

/**
 * Thrown in the case of a group repository member is the read-only policy.
 *
 * @since 3.30
 */
public class GroupWriteDenyException
    extends RuntimeException
{
  private static final long serialVersionUID = -2446385663439609576L;

  public GroupWriteDenyException(String message) {
    super(message);
  }
}
