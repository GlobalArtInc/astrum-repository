
package org.globalart.astrum.security.user;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * {@link UniqueUserId} validator.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class UniqueUserIdValidator
    extends ConstraintValidatorSupport<UniqueUserId, String>
{
  private final UserManager userManager;

  @Autowired
  public UniqueUserIdValidator(final SecuritySystem securitySystem) throws NoSuchUserManagerException {
    this.userManager = checkNotNull(securitySystem).getUserManager(UserManager.DEFAULT_SOURCE);
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    log.trace("Validating unique user-id: {}", value);
    try {
      userManager.getUser(value);
      return false;
    }
    catch (UserNotFoundException e) {
      return true;
    }
  }
}
