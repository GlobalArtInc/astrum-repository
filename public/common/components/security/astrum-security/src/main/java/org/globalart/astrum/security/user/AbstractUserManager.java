
package org.globalart.astrum.security.user;

import java.util.Collection;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.Set;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.security.role.RoleIdentifier;

import com.google.common.collect.Sets;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * An abstract UserManager that handles filtering UserSearchCriteria in memory, this can be used in addition to an
 * external query ( if all the parameters can not be pased to the external source).
 *
 * @author Brian Demers
 */
public abstract class AbstractUserManager
    implements UserManager
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  protected final EventManager eventManager;

  public AbstractUserManager(final EventManager eventManager) {
    this.eventManager = eventManager;
  }

  /**
   * Delegates to {@link #doAddUser(User, String)} and then posts a {@link UserCreatedEvent}.
   *
   * @param user the user to add
   * @param password the user's password
   * @return the added user
   */
  @Override
  public final User addUser(final User user, final String password) {
    User result = doAddUser(user, password);
    eventManager.post(new UserCreatedEvent(result));
    return result;
  }

  public abstract User doAddUser(final User user, final String password);

  /**
   * Delegates to {@link #doUpdateUser(User)} and then posts a {@link UserUpdatedEvent}.
   *
   * @param user the user to update
   * @return the updated user
   * @throws UserNotFoundException if the user does not exist
   */
  @Override
  public final User updateUser(final User user) throws UserNotFoundException {
    User result = doUpdateUser(user);
    eventManager.post(new UserUpdatedEvent(user));
    return result;
  }

  public abstract User doUpdateUser(final User user) throws UserNotFoundException;

  /**
   *
   * @param userId
   * @throws UserNotFoundException
   */
  @Override
  public final void deleteUser(final String userId) throws UserNotFoundException {
    User result = doDeleteUser(userId);
    eventManager.post(new UserDeletedEvent(result));
  }

  public abstract User doDeleteUser(final String userId) throws UserNotFoundException;

  protected Set<User> filterListInMemory(final Set<User> users, final UserSearchCriteria criteria) {
    // use LinkedHashSet to avoid changing sort order
    Set<User> result = new LinkedHashSet<>();

    for (User user : users) {
      if (userMatchesCriteria(user, criteria)) {
        // add the user if it matches the search criteria
        result.add(user);
      }
    }

    return result;
  }

  protected boolean userMatchesCriteria(final User user, final UserSearchCriteria criteria) {
    Set<String> userRoles = new HashSet<>();
    if (user.getRoles() != null) {
      for (RoleIdentifier roleIdentifier : user.getRoles()) {
        userRoles.add(roleIdentifier.getRoleId());
      }
    }

    return matchesCriteria(user.getUserId(), user.getSource(), userRoles, criteria);
  }

  protected boolean matchesCriteria(
      final String userId,
      final String userSource,
      final Collection<String> usersRoles,
      final UserSearchCriteria criteria)
  {
    if (!Strings2.isBlank(criteria.getUserId())
        && !userId.toLowerCase().startsWith(criteria.getUserId().toLowerCase())) {
      return false;
    }

    if (criteria.getSource() != null && !criteria.getSource().equals(userSource)) {
      return false;
    }

    if (criteria.getOneOfRoleIds() != null && !criteria.getOneOfRoleIds().isEmpty()) {
      Set<String> userRoles = new HashSet<>();
      if (usersRoles != null) {
        userRoles.addAll(usersRoles);
      }

      // check the intersection of the roles
      if (Sets.intersection(criteria.getOneOfRoleIds(), userRoles).isEmpty()) {
        return false;
      }
    }

    return true;
  }
}
