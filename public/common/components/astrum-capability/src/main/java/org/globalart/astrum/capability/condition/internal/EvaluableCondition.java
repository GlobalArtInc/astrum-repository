
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityContextAware;
import org.globalart.astrum.capability.CapabilityEvent;
import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.capability.Evaluable;
import org.globalart.astrum.capability.condition.ConditionSupport;
import org.globalart.astrum.common.event.EventManager;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * A condition that delegates to provided {@link Evaluable} for checking if the condition is satisfied.
 * {@link Evaluable#isSatisfied()} is reevaluated after each update of capability the condition is used for.
 *
 * @since capabilities 2.2
 */
public class EvaluableCondition
    extends ConditionSupport
    implements CapabilityContextAware
{

  private CapabilityIdentity capabilityIdentity;

  private final Evaluable evaluable;

  public EvaluableCondition(
      final EventManager eventManager,
      final Evaluable evaluable)
  {
    super(eventManager, false);
    this.evaluable = checkNotNull(evaluable);
  }

  @Override
  public EvaluableCondition setContext(final CapabilityContext context) {
    checkState(!isActive(), "Cannot contextualize when already bounded");
    checkState(capabilityIdentity == null, "Already contextualized with id '" + capabilityIdentity + "'");
    capabilityIdentity = context.id();

    return this;
  }

  @Override
  protected void doBind() {
    checkState(capabilityIdentity != null, "Capability identity not specified");
    getEventManager().register(this);
    setSatisfied(evaluable.isSatisfied());
  }

  @Override
  public void doRelease() {
    getEventManager().unregister(this);
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.AfterUpdate event) {
    if (event.getReference().context().id().equals(capabilityIdentity)) {
      setSatisfied(evaluable.isSatisfied());
    }
  }

  @Override
  public String toString() {
    return evaluable.toString();
  }

  @Override
  public String explainSatisfied() {
    return evaluable.explainSatisfied();
  }

  @Override
  public String explainUnsatisfied() {
    return evaluable.explainUnsatisfied();
  }

}
