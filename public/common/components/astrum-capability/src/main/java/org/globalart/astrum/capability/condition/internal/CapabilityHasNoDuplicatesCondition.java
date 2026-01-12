
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityContextAware;
import org.globalart.astrum.capability.CapabilityEvent;
import org.globalart.astrum.capability.condition.ConditionSupport;
import org.globalart.astrum.common.event.EventManager;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkState;

/**
 * A condition that is satisfied as long as the capability has no duplicates.
 *
 * @since 3.13
 */
public class CapabilityHasNoDuplicatesCondition
    extends ConditionSupport
    implements CapabilityContextAware
{
  private CapabilityContext context;

  public CapabilityHasNoDuplicatesCondition(final EventManager eventManager) {
    super(eventManager);
  }

  @Override
  public CapabilityHasNoDuplicatesCondition setContext(final CapabilityContext context) {
    checkState(!isActive(), "Cannot contextualize when already bounded");
    checkState(this.context == null, "Already contextualized with '" + this.context + "'");
    this.context = context;

    return this;
  }

  @Override
  protected void doBind() {
    checkState(context != null, "Not yet contextualized");
    checkForDuplicates();
    getEventManager().register(this);
  }

  @Override
  public void doRelease() {
    getEventManager().unregister(this);
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.Created event) {
    checkForDuplicates(event);
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.AfterUpdate event) {
    checkForDuplicates(event);
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.AfterRemove event) {
    checkForDuplicates(event);
  }

  private void checkForDuplicates(final CapabilityEvent event) {
    if (event.getReference().context().type().equals(context.type())) {
      checkForDuplicates();
    }
  }

  private void checkForDuplicates() {
    setSatisfied(!context.descriptor().isDuplicated(context.id(), context.properties()));
  }

  @Override
  public String toString() {
    return "Has no duplicates: " + context.id();
  }

  @Override
  public String explainSatisfied() {
    return "Capability has no duplicates";
  }

  @Override
  public String explainUnsatisfied() {
    return "Capability is duplicated by another capability";
  }

}
