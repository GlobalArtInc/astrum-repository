
package org.globalart.astrum.capability.condition.internal;

import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.common.event.EventManager;

/**
 * A condition that applies a logical OR between conditions.
 *
 * @since capabilities 2.0
 */
public class DisjunctionCondition
    extends CompositeConditionSupport
    implements Condition
{

  private Condition lastSatisfied;

  public DisjunctionCondition(
      final EventManager eventManager,
      final Condition... conditions)
  {
    super(eventManager, conditions);
  }

  @Override
  protected boolean reevaluate(final Condition... conditions) {
    for (final Condition condition : conditions) {
      if (condition.isSatisfied()) {
        lastSatisfied = condition;
        return true;
      }
    }
    lastSatisfied = null;
    return false;
  }

  @Override
  public String toString() {
    final StringBuilder sb = new StringBuilder();
    for (final Condition condition : getConditions()) {
      if (sb.length() > 0) {
        sb.append(" OR ");
      }
      sb.append(condition);
    }
    return sb.toString();
  }

  @Override
  public String explainSatisfied() {
    if (lastSatisfied != null) {
      return lastSatisfied.explainSatisfied();
    }
    final StringBuilder sb = new StringBuilder();
    for (final Condition condition : getConditions()) {
      if (sb.length() > 0) {
        sb.append(" OR ");
      }
      sb.append(condition.explainSatisfied());
    }
    return sb.toString();
  }

  @Override
  public String explainUnsatisfied() {
    final StringBuilder sb = new StringBuilder();
    for (final Condition condition : getConditions()) {
      if (sb.length() > 0) {
        sb.append(" AND ");
      }
      sb.append(condition.explainUnsatisfied());
    }
    return sb.toString();
  }

}
