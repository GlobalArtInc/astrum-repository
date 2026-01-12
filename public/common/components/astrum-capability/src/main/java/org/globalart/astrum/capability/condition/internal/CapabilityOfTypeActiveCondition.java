
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityEvent;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.common.event.EventManager;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

/**
 * A condition that is satisfied when a capability of a specified type exists and is in an active state.
 *
 * @since capabilities 2.0
 */
public class CapabilityOfTypeActiveCondition
    extends CapabilityOfTypeExistsCondition
{

  public CapabilityOfTypeActiveCondition(
      final EventManager eventManager,
      final CapabilityDescriptorRegistry descriptorRegistry,
      final CapabilityRegistry capabilityRegistry,
      final CapabilityType type)
  {
    super(eventManager, descriptorRegistry, capabilityRegistry, type);
  }

  @Override
  boolean isSatisfiedBy(final CapabilityReference reference) {
    return super.isSatisfiedBy(reference) && reference.context().isActive();
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.AfterActivated event) {
    if (!isSatisfied() && type.equals(event.getReference().context().type())) {
      checkAllCapabilities();
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.BeforePassivated event) {
    if (isSatisfied() && type.equals(event.getReference().context().type())) {
      checkAllCapabilities();
    }
  }

  @Override
  public String toString() {
    return "Active " + type;
  }

  @Override
  public String explainSatisfied() {
    return typeName + " is active";
  }

  @Override
  public String explainUnsatisfied() {
    return typeName + " is not active";
  }

}
