
package org.globalart.astrum.selector;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.validation.ConstraintValidatorSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * {@link UniqueSelectorName} validator.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class UniqueSelectorNameValidator
    extends ConstraintValidatorSupport<UniqueSelectorName, String>
{
  private final SelectorManager selectorManager;

  @Autowired
  public UniqueSelectorNameValidator(final SelectorManager selectorManager) {
    this.selectorManager = checkNotNull(selectorManager);
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    log.trace("Validating unique selector name: {}", value);
    for (SelectorConfiguration configuration : selectorManager.browse()) {
      if (value.equals(configuration.getName())) {
        return false;
      }
    }
    return true;
  }
}
