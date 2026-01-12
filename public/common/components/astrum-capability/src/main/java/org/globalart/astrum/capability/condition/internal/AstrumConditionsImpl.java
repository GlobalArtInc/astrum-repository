
package org.globalart.astrum.capability.condition.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.condition.AstrumConditions;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default implementation of {@link AstrumConditions}.
 *
 * @since capabilities 2.0
 */
@Component
public class AstrumConditionsImpl
    implements AstrumConditions
{
  private final AstrumIsActiveCondition astrumIsActiveCondition;

  @Autowired
  public AstrumConditionsImpl(final AstrumIsActiveCondition astrumIsActiveCondition) {
    this.astrumIsActiveCondition = checkNotNull(astrumIsActiveCondition);
  }

  @Override
  public Condition active() {
    return astrumIsActiveCondition;
  }
}
