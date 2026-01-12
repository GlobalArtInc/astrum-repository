
package org.globalart.astrum.capability.condition;

import org.globalart.astrum.capability.Condition;

/**
 * Factory of logical {@link Condition}s.
 *
 * @since capabilities 2.0
 */
public interface LogicalConditions
{
  /**
   * Creates a new condition that is satisfied when both conditions are satisfied (logical AND).
   *
   * @param conditions to be AND-ed
   * @return created condition
   */
  Condition and(Condition... conditions);

  /**
   * Creates a new condition that is satisfied when at least one condition is satisfied (logical OR).
   *
   * @param conditions to be OR-ed
   * @return created condition
   */
  Condition or(Condition... conditions);

  /**
   * Creates a new condition that is satisfied when at another condition is not satisfied (logical NOT).
   *
   * @param condition negated condition
   * @return created condition
   */
  Condition not(Condition condition);
}
