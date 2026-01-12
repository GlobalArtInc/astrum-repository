
package org.globalart.astrum.capability.condition.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.Evaluable;
import org.globalart.astrum.capability.condition.CapabilityConditions;
import org.globalart.astrum.common.event.EventManager;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default implementation of {@link CapabilityConditions}.
 *
 * @since capabilities 2.0
 */
@Component
public class CapabilityConditionsImpl
    implements CapabilityConditions
{

  private final CapabilityRegistry capabilityRegistry;

  private final EventManager eventManager;

  private final CapabilityDescriptorRegistry descriptorRegistry;

  @Autowired
  public CapabilityConditionsImpl(
      final EventManager eventManager,
      final CapabilityDescriptorRegistry descriptorRegistry,
      final CapabilityRegistry capabilityRegistry)
  {
    this.descriptorRegistry = checkNotNull(descriptorRegistry);
    this.capabilityRegistry = checkNotNull(capabilityRegistry);
    this.eventManager = checkNotNull(eventManager);
  }

  @Override
  public Condition capabilityOfTypeExists(final CapabilityType type) {
    return new CapabilityOfTypeExistsCondition(eventManager, descriptorRegistry, capabilityRegistry, type);
  }

  @Override
  public Condition capabilityOfTypeActive(final CapabilityType type) {
    return new CapabilityOfTypeActiveCondition(eventManager, descriptorRegistry, capabilityRegistry, type);
  }

  @Override
  public Condition passivateCapabilityDuringUpdate() {
    return new PassivateCapabilityDuringUpdateCondition(eventManager);
  }

  @Override
  public Condition passivateCapabilityWhenPropertyChanged(final String... propertyNames) {
    return new PassivateCapabilityDuringUpdateCondition(eventManager, propertyNames);
  }

  @Override
  public Condition capabilityHasNoFailures() {
    return new CapabilityHasNoFailuresCondition(eventManager);
  }

  @Override
  public Condition evaluable(final Evaluable condition) {
    return new EvaluableCondition(eventManager, condition);
  }

  @Override
  public Condition capabilityHasNoDuplicates() {
    return new CapabilityHasNoDuplicatesCondition(eventManager);
  }
}
