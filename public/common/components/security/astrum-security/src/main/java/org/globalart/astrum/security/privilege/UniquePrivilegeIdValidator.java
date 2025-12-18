
package org.globalart.astrum.security.privilege;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * {@link UniquePrivilegeId} validator.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class UniquePrivilegeIdValidator
    extends ConstraintValidatorSupport<UniquePrivilegeId, String>
{
  private final SecuritySystem securitySystem;

  @Autowired
  public UniquePrivilegeIdValidator(final SecuritySystem securitySystem) {
    this.securitySystem = checkNotNull(securitySystem);
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    log.trace("Validating unique privilege-id: {}", value);
    for (Privilege privilege : securitySystem.listPrivileges()) {
      if (value.equals(privilege.getId())) {
        return false;
      }
    }
    return true;
  }
}
