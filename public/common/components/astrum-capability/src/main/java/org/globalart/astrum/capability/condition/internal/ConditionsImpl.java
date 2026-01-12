
package org.globalart.astrum.capability.condition.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.condition.CapabilityConditions;
import org.globalart.astrum.capability.condition.Conditions;
import org.globalart.astrum.capability.condition.LogicalConditions;
import org.globalart.astrum.capability.condition.AstrumConditions;

import org.springframework.context.annotation.Lazy;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default implementation of {@link Conditions}.
 *
 * @since capabilities 2.0
 */
@Component
public class ConditionsImpl
    implements Conditions
{

  private final LogicalConditions logicalConditions;

  private final CapabilityConditions capabilityConditions;

  private final AstrumConditions astrumConditions;

  @Autowired
  public ConditionsImpl(
      @Lazy final LogicalConditions logicalConditions,
      @Lazy final CapabilityConditions capabilityConditions,
      @Lazy final AstrumConditions astrumConditions)
  {
    this.logicalConditions = checkNotNull(logicalConditions);
    this.capabilityConditions = checkNotNull(capabilityConditions);
    this.astrumConditions = checkNotNull(astrumConditions);
  }

  @Override
  public LogicalConditions logical() {
    return logicalConditions;
  }

  @Override
  public CapabilityConditions capabilities() {
    return capabilityConditions;
  }

  @Override
  public AstrumConditions astrum() {
    return astrumConditions;
  }

  @Override
  public Condition always(final String reason) {
    return new SatisfiedCondition(reason);
  }

  @Override
  public Condition never(final String reason) {
    return new UnsatisfiedCondition(reason);
  }
}
