
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.common.event.EventManager;

/**
 * A condition that applies a logical NOT on another condition.
 *
 * @since capabilities 2.0
 */
public class InversionCondition
    extends CompositeConditionSupport
    implements Condition
{

  private final Condition condition;

  public InversionCondition(
      final EventManager eventManager,
      final Condition condition)
  {
    super(eventManager, condition);
    this.condition = condition;
  }

  @Override
  protected boolean reevaluate(final Condition... conditions) {
    return !conditions[0].isSatisfied();
  }

  @Override
  public String toString() {
    return "NOT " + condition;
  }

  @Override
  public String explainSatisfied() {
    return condition.explainUnsatisfied();
  }

  @Override
  public String explainUnsatisfied() {
    return condition.explainSatisfied();
  }
}
