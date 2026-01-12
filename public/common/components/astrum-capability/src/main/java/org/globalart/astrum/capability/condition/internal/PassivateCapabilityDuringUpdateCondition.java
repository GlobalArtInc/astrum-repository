
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityContextAware;
import org.globalart.astrum.capability.CapabilityEvent;
import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.capability.condition.ConditionSupport;
import org.globalart.astrum.common.event.EventManager;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkState;

/**
 * A condition that is becoming unsatisfied before an capability is updated and becomes satisfied after capability was
 * updated.
 *
 * @since capabilities 2.0
 */
public class PassivateCapabilityDuringUpdateCondition
    extends ConditionSupport
    implements CapabilityContextAware
{

  private CapabilityIdentity id;

  private final String[] propertyNames;

  public PassivateCapabilityDuringUpdateCondition(
      final EventManager eventManager,
      final String... propertyNames)
  {
    super(eventManager, true);
    this.propertyNames = propertyNames == null || propertyNames.length == 0 ? null : propertyNames;
  }

  @Override
  public PassivateCapabilityDuringUpdateCondition setContext(final CapabilityContext context) {
    checkState(!isActive(), "Cannot contextualize when already bounded");
    checkState(id == null, "Already contextualized with id '" + id + "'");
    id = context.id();

    return this;
  }

  @Override
  protected void doBind() {
    checkState(id != null, "Capability identity not specified");
    getEventManager().register(this);
  }

  @Override
  public void doRelease() {
    getEventManager().unregister(this);
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.BeforeUpdate event) {
    if (event.getReference().context().id().equals(id)) {
      if (propertyNames == null) {
        setSatisfied(false);
      }
      else {
        for (final String propertyName : propertyNames) {
          String oldValue = event.properties().get(propertyName);
          if (oldValue == null) {
            oldValue = "";
          }
          String newValue = event.previousProperties().get(propertyName);
          if (newValue == null) {
            newValue = "";
          }
          if (isSatisfied() && !oldValue.equals(newValue)) {
            setSatisfied(false);
          }
        }
      }
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.AfterUpdate event) {
    if (event.getReference().context().id().equals(id)) {
      setSatisfied(true);
    }
  }

  @Override
  public String toString() {
    return "Passivate during update of " + id;
  }

  @Override
  public String explainSatisfied() {
    return "Capability is currently being updated";
  }

  @Override
  public String explainUnsatisfied() {
    return "Capability is not currently being updated";
  }

}
