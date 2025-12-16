
package org.globalart.astrum.security;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.security.user.NoSuchUserManagerException;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserManager;
import org.globalart.astrum.security.user.UserNotFoundException;
import org.globalart.astrum.security.user.UserNotFoundTransientException;
import org.globalart.astrum.security.user.UserStatus;

import org.apache.shiro.subject.PrincipalCollection;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Helper component to map user principals to associated information.
 */
@Component
public class UserPrincipalsHelper
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final List<UserManager> userManagers;

  @Autowired
  public UserPrincipalsHelper(final List<UserManager> userManagers) {
    this.userManagers = checkNotNull(userManagers);
  }

  /**
   * Searches records to find the status of the user associated with the given principals.
   *
   * @param principals Identifying principals
   * @return User status
   */
  public UserStatus getUserStatus(final PrincipalCollection principals) throws UserNotFoundException {
    String userId = null;
    if (principals != null) {
      userId = principals.getPrimaryPrincipal().toString();
      try {
        final User user = findUserManager(principals).getUser(userId);
        if (user != null) {
          return user.getStatus();
        }
      }
      catch (final NoSuchUserManagerException e) {
        throw new UserNotFoundException(userId, e.getMessage(), e);
      }
      catch (final UserNotFoundTransientException e) {
        log.debug("Ignoring transient user error", e);
        return UserStatus.disabled;
      }
      catch (final UserNotFoundException e) {
        throw e; // pass back original cause unchanged
      }
      catch (final RuntimeException e) {
        log.debug("Ignoring transient user error", e);
        return UserStatus.disabled;
      }
    }
    throw new UserNotFoundException(userId);
  }

  /**
   * Searches for the {@link UserManager} associated with the given principals.
   *
   * @param principals Identifying principals
   * @return UserManager component
   */
  public UserManager findUserManager(final PrincipalCollection principals) throws NoSuchUserManagerException {
    if (principals == null) {
      throw new NoSuchUserManagerException("Missing principals");
    }
    boolean isPrimary = true;
    for (final String realmName : principals.getRealmNames()) {
      // include secondary realms in the search as long as they have the same userId as the primary
      if (isPrimary || principals.fromRealm(realmName).contains(principals.getPrimaryPrincipal())) {
        for (final UserManager userManager : userManagers) {
          if (realmName.equals(userManager.getAuthenticationRealmName())) {
            return userManager;
          }
        }
        isPrimary = false;
      }
    }
    throw new NoSuchUserManagerException("User-manager not found for realm(s)", principals.getRealmNames().toString());
  }
}
