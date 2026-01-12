
package org.globalart.astrum.capability.condition;

import org.globalart.astrum.capability.Condition;

/**
 * Central access point for built in {@link Condition}s.
 *
 * @since capabilities 2.0
 */
public interface Conditions
{
  /**
   * Access to logical conditions.
   *
   * @return logical conditions factory
   */
  LogicalConditions logical();

  /**
   * Access to capability related conditions.
   *
   * @return capability related conditions factory
   */
  CapabilityConditions capabilities();

  /**
   * Access to astrum specific conditions.
   *
   * @return astrum specific conditions factory
   */
  AstrumConditions astrum();

  /**
   * Creates a new condition that is always satisfied for the specified reason.
   *
   * @since 3.1
   */
  Condition always(String reason);

  /**
   * Creates a new condition that is never satisfied for the specified reason.
   * 
   * @since 3.1
   */
  Condition never(String reason);
}
