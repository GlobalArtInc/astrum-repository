
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
 * {@link UserExists} validator.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class UserExistsValidator
    extends ConstraintValidatorSupport<UserExists, String>
{
  private final SecuritySystem securitySystem;

  @Autowired
  public UserExistsValidator(final SecuritySystem securitySystem) {
    this.securitySystem = checkNotNull(securitySystem);
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    log.trace("Validating user exists: {}", value);
    try {
      User user = securitySystem.getUser(value);
      return true;
    }
    catch (UserNotFoundException e) {
      return false;
    }
  }
}
