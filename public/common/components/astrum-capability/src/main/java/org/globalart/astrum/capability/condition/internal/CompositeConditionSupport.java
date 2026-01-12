
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityContextAware;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.ConditionEvent;
import org.globalart.astrum.capability.condition.ConditionSupport;
import org.globalart.astrum.common.event.EventManager;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Composite {@link Condition} implementation support.
 *
 * @since capabilities 2.0
 */
public abstract class CompositeConditionSupport
    extends ConditionSupport
    implements CapabilityContextAware
{

  private final Condition[] conditions;

  public CompositeConditionSupport(
      final EventManager eventManager,
      final Condition... conditions)
  {
    super(eventManager, false);
    this.conditions = checkNotNull(conditions);
    checkArgument(conditions.length > 1, "A composite mush have at least 2 conditions");
  }

  public CompositeConditionSupport(
      final EventManager eventManager,
      final Condition condition)
  {
    super(eventManager, false);
    this.conditions = new Condition[]{checkNotNull(condition)};
  }

  @Override
  protected void doBind() {
    for (final Condition condition : conditions) {
      condition.bind();
    }
    getEventManager().register(this);
    setSatisfied(reevaluate(conditions));
  }

  @Override
  public void doRelease() {
    getEventManager().unregister(this);
    for (final Condition condition : conditions) {
      condition.release();
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final ConditionEvent.Satisfied event) {
    if (shouldReevaluateFor(event.getCondition())) {
      setSatisfied(reevaluate(conditions));
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final ConditionEvent.Unsatisfied event) {
    if (shouldReevaluateFor(event.getCondition())) {
      setSatisfied(reevaluate(conditions));
    }
  }

  @Override
  public CompositeConditionSupport setContext(final CapabilityContext context) {
    for (final Condition condition : conditions) {
      if (condition instanceof CapabilityContextAware) {
        ((CapabilityContextAware) condition).setContext(context);
      }
    }
    return this;
  }

  @Override
  public String toString() {
    return "Re-evaluate " + CompositeConditionSupport.this;
  }

  /**
   * Whether or not the composite conditions are satisfied as a unit.
   *
   * @param conditions to be checked (there are at least 2 conditions passed in)
   * @return true, if conditions are satisfied as a unit
   */
  protected abstract boolean reevaluate(final Condition... conditions);

  protected Condition[] getConditions() {
    return conditions;
  }

  private boolean shouldReevaluateFor(final Condition condition) {
    for (final Condition watched : conditions) {
      if (watched == condition) {
        return true;
      }
    }
    return false;
  }

}
