
package org.globalart.astrum.capability;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.validation.ConstraintValidatorSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * {@link CapabilityTypeExists} validator.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CapabilityTypeExistsValidator
    extends ConstraintValidatorSupport<CapabilityTypeExists, String>
{
  private final CapabilityFactoryRegistry capabilityFactoryRegistry;

  private final CapabilityDescriptorRegistry capabilityDescriptorRegistry;

  @Autowired
  public CapabilityTypeExistsValidator(
      final CapabilityFactoryRegistry capabilityFactoryRegistry,
      final CapabilityDescriptorRegistry capabilityDescriptorRegistry)
  {
    this.capabilityFactoryRegistry = checkNotNull(capabilityFactoryRegistry);
    this.capabilityDescriptorRegistry = checkNotNull(capabilityDescriptorRegistry);
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    if (value != null) {
      log.trace("Validating capability type exists: {}", value);
      CapabilityType type = CapabilityType.capabilityType(value);
      return capabilityFactoryRegistry.get(type) != null && capabilityDescriptorRegistry.get(type) != null;
    }
    return true;
  }
}
