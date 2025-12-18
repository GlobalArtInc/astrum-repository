
package org.globalart.astrum.security.realm;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.validation.ConstraintValidatorSupport;

import org.apache.shiro.mgt.RealmSecurityManager;
import org.apache.shiro.realm.Realm;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * {@link RealmExists} validator.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RealmExistsValidator
    extends ConstraintValidatorSupport<RealmExists, String>
{
  private final RealmSecurityManager realmSecurityManager;

  @Autowired
  public RealmExistsValidator(final RealmSecurityManager realmSecurityManager) {
    this.realmSecurityManager = checkNotNull(realmSecurityManager);
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    log.trace("Validating realm exists: {}", value);
    for (Realm realm : realmSecurityManager.getRealms()) {
      if (value.equals(realm.getName())) {
        return true;
      }
    }
    return false;
  }
}
