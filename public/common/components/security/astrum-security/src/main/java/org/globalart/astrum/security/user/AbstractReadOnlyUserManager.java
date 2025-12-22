
package org.globalart.astrum.security.user;

import org.globalart.astrum.common.event.EventManager;

/**
 * Read-only {@link UserManager}, which just throws exceptions for all the write methods.
 *
 * Any call to theses methods should be guarded by {@code #supportsWrite}.
 */
public abstract class AbstractReadOnlyUserManager
    extends AbstractUserManager
{
  public AbstractReadOnlyUserManager(EventManager eventManager) {
    super(eventManager);
  }

  /**
   * @return Always {@code false}
   */
  @Override
  public boolean supportsWrite() {
    return false;
  }

  @Override
  public User doAddUser(final User user, final String password) {
    throw new UnsupportedOperationException();
  }

  @Override
  public void changePassword(final String userId, final String newPassword) throws UserNotFoundException {
    throw new UnsupportedOperationException();
  }

  @Override
  public User doDeleteUser(final String userId) throws UserNotFoundException {
    throw new UnsupportedOperationException();
  }

  @Override
  public User doUpdateUser(final User user) throws UserNotFoundException {
    throw new UnsupportedOperationException();
  }
}
