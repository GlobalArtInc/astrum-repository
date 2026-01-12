
package org.globalart.astrum.capability;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintValidatorContext;

import org.globalart.astrum.validation.ConstraintValidatorSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * {@link CapabilityTypeIsExposed} validator. Note that this validator also validates the CapabilityTypeExists
 * so there is no need to utilize both validators for single CapabilityType
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CapabilityTypeIsExposedValidator
    extends ConstraintValidatorSupport<CapabilityTypeIsExposed, String>
{
  private final CapabilityFactoryRegistry capabilityFactoryRegistry;

  private final CapabilityDescriptorRegistry capabilityDescriptorRegistry;

  @Autowired
  public CapabilityTypeIsExposedValidator(
      final CapabilityFactoryRegistry capabilityFactoryRegistry,
      final CapabilityDescriptorRegistry capabilityDescriptorRegistry)
  {
    this.capabilityFactoryRegistry = checkNotNull(capabilityFactoryRegistry);
    this.capabilityDescriptorRegistry = checkNotNull(capabilityDescriptorRegistry);
  }

  @Override
  public boolean isValid(final String value, final ConstraintValidatorContext context) {
    if (value != null) {
      log.trace("Validating capability type is exposed: {}", value);
      CapabilityType type = CapabilityType.capabilityType(value);
      CapabilityDescriptor capabilityDescriptor = capabilityDescriptorRegistry.get(type);
      return capabilityFactoryRegistry.get(type) != null && capabilityDescriptor != null
          && capabilityDescriptor.isExposed();
    }
    return true;
  }
}
