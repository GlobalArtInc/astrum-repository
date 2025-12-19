
package org.globalart.astrum.security.role;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.authz.AuthorizationManager;
import org.globalart.astrum.security.authz.NoSuchAuthorizationManagerException;
import org.globalart.astrum.security.internal.AuthorizationManagerImpl;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * {@link UniqueRoleId} validator.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class UniqueRoleIdValidator
    extends ConstraintValidatorSupport<UniqueRoleId, String>
{
  private final AuthorizationManager authorizationManager;

  @Autowired
  public UniqueRoleIdValidator(final SecuritySystem securitySystem) throws NoSuchAuthorizationManagerException {
    this.authorizationManager = checkNotNull(securitySystem).getAuthorizationManager(AuthorizationManagerImpl.SOURCE);
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    log.trace("Validating unique role-id: {}", value);
    try {
      authorizationManager.getRole(value);
      return false;
    }
    catch (NoSuchRoleException e) {
      return true;
    }
  }
}
