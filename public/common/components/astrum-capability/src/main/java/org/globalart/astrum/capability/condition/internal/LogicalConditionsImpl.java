
package org.globalart.astrum.capability.condition.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.condition.LogicalConditions;
import org.globalart.astrum.common.event.EventManager;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default implementation of {@link LogicalConditions}.
 *
 * @since capabilities 2.0
 */
@Component
public class LogicalConditionsImpl
    implements LogicalConditions
{

  private final EventManager eventManager;

  @Autowired
  public LogicalConditionsImpl(final EventManager eventManager) {
    this.eventManager = checkNotNull(eventManager);
  }

  @Override
  public Condition and(final Condition... conditions) {
    return new ConjunctionCondition(eventManager, conditions);
  }

  @Override
  public Condition or(final Condition... conditions) {
    return new DisjunctionCondition(eventManager, conditions);
  }

  @Override
  public Condition not(final Condition condition) {
    return new InversionCondition(eventManager, condition);
  }
}
