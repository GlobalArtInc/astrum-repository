
package org.globalart.astrum.capability.condition.internal;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.common.lifecycle.Lifecycle;
import org.globalart.astrum.capability.condition.ConditionSupport;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventManager;

import com.google.common.annotations.VisibleForTesting;

import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.CAPABILITIES;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

/**
 * A condition that is satisfied when astrum is active.
 *
 * @since capabilities 2.0
 */
@Component
@ManagedLifecycle(phase = CAPABILITIES)
@Order(Ordered.LOWEST_PRECEDENCE) // astrum is considered active at the end of this phase
public class AstrumIsActiveCondition
    extends ConditionSupport
    implements Lifecycle
{
  @VisibleForTesting
  public AstrumIsActiveCondition(final EventManager eventManager) {
    super(eventManager, false);
    bind();
  }

  @Autowired
  public AstrumIsActiveCondition(final Provider<EventManager> eventManager) {
    super(eventManager, false);
    bind();
  }

  @Override
  public void start() {
    setSatisfied(true);
  }

  @Override
  public void stop() {
    setSatisfied(false);
  }

  @Override
  protected void doBind() {
    // do nothing
  }

  @Override
  protected void doRelease() {
    // do nothing
  }

  @Override
  public String toString() {
    return "Astrum is active";
  }

  @Override
  public String explainSatisfied() {
    return "Astrum is active";
  }

  @Override
  public String explainUnsatisfied() {
    return "Astrum is not active";
  }
}
