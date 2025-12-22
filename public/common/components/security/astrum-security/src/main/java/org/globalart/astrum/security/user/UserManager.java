
package org.globalart.astrum.security.user;

import java.util.Set;

import org.apache.shiro.realm.Realm;

/**
 * User manager.
 */
public interface UserManager
{
  String DEFAULT_SOURCE = "default";

  String SAML_SOURCE = "SAML";

  /**
   * Get the source string of this UserManager
   */
  String getSource();

  /**
   * The name of the {@link Realm} is associated with.
   */
  String getAuthenticationRealmName();

  /**
   * If this UserManager is writable.
   */
  boolean supportsWrite();

  /**
   * Retrieve all User objects
   */
  Set<User> listUsers();

  // FIXME: This is only used by tests
  /**
   * Retrieve all user ids.
   */
  Set<String> listUserIds();

  /**
   * Add a user.
   */
  User addUser(User user, String password);

  /**
   * Update a user.
   */
  User updateUser(User user) throws UserNotFoundException;

  /**
   * Delete a user based on id.
   */
  void deleteUser(String userId) throws UserNotFoundException;

  /**
   * Searches for Subject objects by a criteria.
   */
  Set<User> searchUsers(UserSearchCriteria criteria);

  /**
   * Get a Subject object by id
   */
  User getUser(String userId) throws UserNotFoundException;

  /**
   * Optimized version of getUser when roleIds are known, so we don't fetch them again.
   */
  User getUser(String userId, Set<String> roleIds) throws UserNotFoundException;

  /**
   * Update a users password.
   */
  void changePassword(String userId, String newPassword) throws UserNotFoundException;

  /**
   * @return {@code true} if this realm has a configuration associated with it, {@code false} otherwise
   */
  boolean isConfigured();
}
