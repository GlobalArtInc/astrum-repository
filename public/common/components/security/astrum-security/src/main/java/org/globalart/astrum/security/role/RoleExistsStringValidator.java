
package org.globalart.astrum.security.role;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.security.authz.AuthorizationManager;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.springframework.beans.factory.config.ConfigurableBeanFactory.SCOPE_PROTOTYPE;

@Component
@Scope(SCOPE_PROTOTYPE)
public class RoleExistsStringValidator
    extends ConstraintValidatorSupport<RoleExistsString, String>
{
  private final AuthorizationManager authorizationManager;

  @Autowired
  public RoleExistsStringValidator(final AuthorizationManager authorizationManager) {
    this.authorizationManager = authorizationManager;
  }

  @Override
  public boolean isValid(final String role, final ConstraintValidatorContext constraintValidatorContext) {
    try {
      authorizationManager.getRole(role);
      return true;
    }
    catch (NoSuchRoleException e) {
      return false;
    }
  }
}
