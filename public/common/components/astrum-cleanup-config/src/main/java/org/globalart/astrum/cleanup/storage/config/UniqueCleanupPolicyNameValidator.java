
package org.globalart.astrum.cleanup.storage.config;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.cleanup.storage.CleanupPolicyStorage;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Ensure that Cleanup Policy names are unique case-insensitively.
 *
 * @since 3.14
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class UniqueCleanupPolicyNameValidator
    extends ConstraintValidatorSupport<UniqueCleanupPolicyName, String>
{
  private CleanupPolicyStorage cleanupPolicyStorage;

  @Autowired
  public UniqueCleanupPolicyNameValidator(final CleanupPolicyStorage cleanupPolicyStorage) {
    this.cleanupPolicyStorage = checkNotNull(cleanupPolicyStorage);
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    return !cleanupPolicyStorage.exists(value);
  }
}
