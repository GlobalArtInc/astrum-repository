
package org.globalart.astrum.internal.capability.internal;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.capability.condition.Conditions;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.internal.capability.ActivationConditionHandler;
import org.globalart.astrum.internal.capability.ActivationConditionHandlerFactory;
import org.globalart.astrum.internal.capability.DefaultCapabilityReference;

import org.springframework.stereotype.Component;

@Component
public class ActivationConditionHandlerFactoryImpl
    implements ActivationConditionHandlerFactory
{
  private final EventManager eventManager;

  private final Conditions conditions;

  @Autowired
  public ActivationConditionHandlerFactoryImpl(final EventManager eventManager, final Conditions conditions) {
    this.eventManager = eventManager;
    this.conditions = conditions;
  }

  @Override
  public ActivationConditionHandler create(final DefaultCapabilityReference reference) {
    return new ActivationConditionHandler(eventManager, conditions, reference);
  }
}
