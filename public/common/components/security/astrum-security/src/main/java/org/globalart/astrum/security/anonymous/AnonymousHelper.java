
package org.globalart.astrum.security.anonymous;

import java.util.List;
import java.util.Objects;
import javax.annotation.Nullable;

import org.globalart.astrum.security.internal.AuthorizingRealmImpl;
import org.globalart.astrum.security.internal.DefaultRealmConstants;
import org.globalart.astrum.security.user.UserManager;

import org.apache.shiro.subject.PrincipalCollection;
import org.apache.shiro.subject.Subject;

import static java.util.stream.Collectors.toList;

/**
 * Anonymous helpers.
 *
 * @since 3.0
 */
public class AnonymousHelper
{
  private AnonymousHelper() {
    // empty
  }

  /**
   * Check given given subject is anonymous.
   */
  public static boolean isAnonymous(@Nullable final Subject subject) {
    return subject != null && subject.getPrincipals() instanceof AnonymousPrincipalCollection;
  }

  /**
   * Check given principals represent anonymous.
   *
   * @since 3.22
   */
  public static boolean isAnonymous(@Nullable final PrincipalCollection principals) {
    return principals instanceof AnonymousPrincipalCollection;
  }

  /**
   * Get all authentication realms.
   */
  public static List<String> getAuthenticationRealms(final List<UserManager> userManagers) {
    return userManagers.stream()
        .map(UserManager::getAuthenticationRealmName)
        .filter(Objects::nonNull)
        .map(realm -> realm.equals(DefaultRealmConstants.DEFAULT_REALM_NAME) ? AuthorizingRealmImpl.NAME : realm)
        .collect(toList());
  }
}
