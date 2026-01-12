
package org.globalart.astrum.capability.condition;

import org.globalart.astrum.capability.Condition;

/**
 * Factory of {@link Condition}s related to Astrum.
 *
 * @since capabilities 2.0
 */
public interface AstrumConditions
{
  /**
   * Creates a new condition that is satisfied when Astrum is active (started but not stopped).
   *
   * @return created condition
   */
  Condition active();
}
