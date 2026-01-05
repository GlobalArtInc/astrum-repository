
package org.globalart.astrum.security.user;

import org.globalart.astrum.common.event.EventManager;

import java.util.Collections;
import java.util.LinkedHashSet;
import java.util.Set;

public abstract class MockUserManagerSupport
    extends AbstractUserManager
{
  private Set<User> users = new LinkedHashSet<>();

  public MockUserManagerSupport(final EventManager eventManager) {
    super(eventManager);
  }

  public boolean supportsWrite() {
    return true;
  }

  public User doAddUser(User user, String password) {
    this.getUsers().add(user);
    return user;
  }

  public User doUpdateUser(User user) throws UserNotFoundException {
    User existingUser = this.getUser(user.getUserId());

    if (existingUser == null) {
      throw new UserNotFoundException(user.getUserId());
    }

    return user;
  }

  public User doDeleteUser(String userId) throws UserNotFoundException {
    User existingUser = this.getUser(userId);

    if (existingUser == null) {
      throw new UserNotFoundException(userId);
    }

    this.getUsers().remove(existingUser);
    return existingUser;
  }

  public User getUser(String userId) {
    for (User user : this.getUsers()) {
      if (user.getUserId().equals(userId)) {
        return user;
      }
    }
    return null;
  }

  @Override
  public User getUser(final String userId, final Set<String> roleIds) throws UserNotFoundException {
    return getUser(userId);
  }

  public Set<String> listUserIds() {
    Set<String> userIds = new LinkedHashSet<>();

    for (User user : this.getUsers()) {
      userIds.add(user.getUserId());
    }

    return userIds;
  }

  public Set<User> listUsers() {
    return Collections.unmodifiableSet(this.getUsers());
  }

  public Set<User> searchUsers(UserSearchCriteria criteria) {
    return this.filterListInMemory(this.getUsers(), criteria);
  }

  protected Set<User> getUsers() {
    return users;
  }

  public void changePassword(String userId, String newPassword) throws UserNotFoundException {
    // empty
  }

  @Override
  public boolean isConfigured() {
    throw new UnsupportedOperationException("Not supported yet.");
  }
}
