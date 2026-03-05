
package org.globalart.astrum.cleanup.storage.config;

import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.cleanup.storage.config.RegexCriteriaValidator.InvalidExpressionException;
import org.globalart.astrum.validation.ConstraintValidatorSupport;

import static java.util.Objects.nonNull;
import static org.globalart.astrum.cleanup.storage.config.RegexCriteriaValidator.validate;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 *
 * @since 3.19
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CleanupPolicyAssetNamePatternValidator
    extends ConstraintValidatorSupport<CleanupPolicyAssetNamePattern, String>
{
  public CleanupPolicyAssetNamePatternValidator() {
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    // we don't require it
    if (nonNull(value)) {

      try {
        // but if it is provided we require it to be valid
        validate(value);
      }
      catch (InvalidExpressionException e) { // NOSONAR
        context.disableDefaultConstraintViolation();
        context.buildConstraintViolationWithTemplate(getEscapeHelper().stripJavaEl(e.getMessage()))
            .addConstraintViolation();
        return false;
      }
    }

    return true;
  }
}
