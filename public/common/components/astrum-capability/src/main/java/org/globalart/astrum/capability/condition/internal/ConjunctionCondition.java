
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.common.event.EventManager;

/**
 * A condition that applies a logical AND between conditions.
 *
 * @since capabilities 2.0
 */
public class ConjunctionCondition
    extends CompositeConditionSupport
    implements Condition
{

  private Condition lastNotSatisfied;

  public ConjunctionCondition(
      final EventManager eventManager,
      final Condition... conditions)
  {
    super(eventManager, conditions);
  }

  @Override
  protected boolean reevaluate(final Condition... conditions) {
    for (final Condition condition : conditions) {
      if (!condition.isSatisfied()) {
        lastNotSatisfied = condition;
        return false;
      }
    }
    lastNotSatisfied = null;
    return true;
  }

  @Override
  public String toString() {
    final StringBuilder sb = new StringBuilder();
    for (final Condition condition : getConditions()) {
      if (sb.length() > 0) {
        sb.append(" AND ");
      }
      sb.append(condition);
    }
    return sb.toString();
  }

  @Override
  public String explainSatisfied() {
    final StringBuilder sb = new StringBuilder();
    for (final Condition condition : getConditions()) {
      if (sb.length() > 0) {
        sb.append(" AND ");
      }
      sb.append(condition.explainSatisfied());
    }
    return sb.toString();
  }

  @Override
  public String explainUnsatisfied() {
    if (lastNotSatisfied != null) {
      return lastNotSatisfied.explainUnsatisfied();
    }
    final StringBuilder sb = new StringBuilder();
    for (final Condition condition : getConditions()) {
      if (sb.length() > 0) {
        sb.append(" OR ");
      }
      sb.append(condition.explainUnsatisfied());
    }
    return sb.toString();
  }
}
