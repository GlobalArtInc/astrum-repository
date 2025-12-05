
package org.globalart.astrum.internal.capability.internal;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.condition.Conditions;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.internal.capability.DefaultCapabilityReference;
import org.globalart.astrum.internal.capability.ValidityConditionHandler;
import org.globalart.astrum.internal.capability.ValidityConditionHandlerFactory;

import org.springframework.context.annotation.Lazy;
import org.springframework.stereotype.Component;

@Component
public class ValidityConditionHandlerFactoryImpl
    implements ValidityConditionHandlerFactory
{
  private final EventManager eventManager;

  private final CapabilityRegistry capabilityRegistry;

  private final Conditions conditions;

  @Autowired
  public ValidityConditionHandlerFactoryImpl(
      final EventManager eventManager,
      @Lazy final CapabilityRegistry capabilityRegistry,
      final Conditions conditions)
  {
    this.eventManager = eventManager;
    this.capabilityRegistry = capabilityRegistry;
    this.conditions = conditions;
  }

  @Override
  public ValidityConditionHandler create(final DefaultCapabilityReference reference) {
    return new ValidityConditionHandler(eventManager, capabilityRegistry, conditions, reference);
  }
}
