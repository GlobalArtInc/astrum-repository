
package org.globalart.astrum.onboarding.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.onboarding.OnboardingItem;
import org.globalart.astrum.onboarding.OnboardingItemPriority;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.internal.UserManagerImpl;
import org.globalart.astrum.security.user.NoSuchUserManagerException;
import org.globalart.astrum.security.user.User;
import org.globalart.astrum.security.user.UserNotFoundException;
import org.globalart.astrum.security.user.UserStatus;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * @since 3.17
 */
@Component
public class ChangeAdminPasswordOnboardingItem
    implements OnboardingItem
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final SecuritySystem securitySystem;

  @Autowired
  public ChangeAdminPasswordOnboardingItem(final SecuritySystem securitySystem) {
    this.securitySystem = checkNotNull(securitySystem);
  }

  @Override
  public String getType() {
    return "ChangeAdminPassword";
  }

  @Override
  public int getPriority() {
    return OnboardingItemPriority.CHANGE_ADMIN_PASSWORD_ONBOARDING;
  }

  @Override
  public boolean applies() {
    try {
      User user = securitySystem.getUser("admin", UserManagerImpl.DEFAULT_SOURCE);
      return UserStatus.changepassword.equals(user.getStatus());
    }
    catch (UserNotFoundException e) {
      log.trace("admin user not found in system, marking onboarding item as not applicable.", e);
    }
    catch (NoSuchUserManagerException e) {
      log.trace("default UserManager not found in system, marking onboarding item as not applicable.", e);
    }

    return false;
  }
}
