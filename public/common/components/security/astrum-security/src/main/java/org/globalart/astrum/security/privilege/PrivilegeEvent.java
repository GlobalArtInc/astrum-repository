
package org.globalart.astrum.security.privilege;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link Privilege} event.
 *
 * @since 3.1
 */
public abstract class PrivilegeEvent
{
  private final Privilege privilege;

  public PrivilegeEvent(final Privilege privilege) {
    this.privilege = checkNotNull(privilege);
  }

  public Privilege getPrivilege() {
    return privilege;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "privilege=" + privilege +
        '}';
  }
}
