
package org.globalart.astrum.security.role;

import java.util.Collection;
import java.util.LinkedList;
import java.util.List;

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
 * {@link RolesExist} validator.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RolesExistValidator
    extends ConstraintValidatorSupport<RolesExist, Collection<?>> // Collection<String> expected
{
  private final AuthorizationManager authorizationManager;

  @Autowired
  public RolesExistValidator(final SecuritySystem securitySystem) throws NoSuchAuthorizationManagerException {
    this.authorizationManager = checkNotNull(securitySystem).getAuthorizationManager(AuthorizationManagerImpl.SOURCE);
  }

  @Override
  public boolean isValid(final Collection<?> value, final ConstraintValidatorContext context) {
    log.trace("Validating roles exist: {}", value);
    List<Object> missing = new LinkedList<>();
    for (Object item : value) {
      try {
        authorizationManager.getRole(String.valueOf(item));
      }
      catch (NoSuchRoleException e) {
        missing.add(getEscapeHelper().stripJavaEl(item.toString()));
      }
    }
    if (missing.isEmpty()) {
      return true;
    }

    context.disableDefaultConstraintViolation();
    context.buildConstraintViolationWithTemplate("Missing roles: " + missing)
        .addConstraintViolation();
    return false;
  }
}
