
package org.globalart.astrum.internal.capability;

import org.globalart.astrum.capability.CapabilityContextAware;
import org.globalart.astrum.capability.Condition;
import org.globalart.astrum.capability.ConditionEvent;
import org.globalart.astrum.capability.condition.Conditions;
import org.globalart.astrum.common.event.EventManager;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.beans.factory.annotation.Autowired;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Handles capability activation by reacting capability activation condition being satisfied/unsatisfied.
 *
 * @since capabilities 2.0
 */
public class ActivationConditionHandler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final EventManager eventManager;

  private final DefaultCapabilityReference reference;

  private final Conditions conditions;

  private Condition activationCondition;

  private Condition capabilityActivationCondition;

  private Condition astrumActiveCondition;

  @Autowired
  public ActivationConditionHandler(
      final EventManager eventManager,
      final Conditions conditions,
      final DefaultCapabilityReference reference)
  {
    this.eventManager = checkNotNull(eventManager);
    this.conditions = checkNotNull(conditions);
    this.reference = checkNotNull(reference);
  }

  boolean isConditionSatisfied() {
    return activationCondition != null && activationCondition.isSatisfied();
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final ConditionEvent.Satisfied event) {
    if (event.getCondition() == activationCondition) {
      reference.activate();
    }
    else if (reference.hasFailure() && isRecoveryTrigger(event.getCondition())) {
      reference.resetFailure();
    }
  }

  private boolean isRecoveryTrigger(final Condition condition) {
    return (capabilityActivationCondition != null && condition == capabilityActivationCondition)
        || (astrumActiveCondition != null && condition == astrumActiveCondition);
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final ConditionEvent.Unsatisfied event) {
    if (event.getCondition() == activationCondition) {
      reference.passivate();
    }
  }

  ActivationConditionHandler bind() {
    if (activationCondition == null) {
      try {
        capabilityActivationCondition = reference.capability().activationCondition();
        if (capabilityActivationCondition == null) {
          capabilityActivationCondition = conditions.always("Capability has no activation condition");
        }
        astrumActiveCondition = conditions.astrum().active();
        activationCondition = conditions.logical()
            .and(
                capabilityActivationCondition,
                astrumActiveCondition,
                conditions.capabilities().capabilityHasNoFailures(),
                conditions.capabilities().capabilityHasNoDuplicates());
        if (activationCondition instanceof CapabilityContextAware) {
          ((CapabilityContextAware) activationCondition).setContext(reference.context());
        }
      }
      catch (Exception e) {
        capabilityActivationCondition = null;
        astrumActiveCondition = null;
        activationCondition = conditions.never("Failed to determine activation condition");
        log.error(
            "Could not get activation condition from capability {} ({}). Considering it as non activatable",
            reference.capability(), reference.context().id(), e);
      }
      activationCondition.bind();
      eventManager.register(this);
    }
    return this;
  }

  ActivationConditionHandler release() {
    if (activationCondition != null) {
      eventManager.unregister(this);
      activationCondition.release();
      activationCondition = null;
      capabilityActivationCondition = null;
      astrumActiveCondition = null;
    }
    return this;
  }

  @Override
  public String toString() {
    return String.format(
        "Watching '%s' condition to activate/passivate capability '%s (id=%s)'",
        activationCondition, reference.capability(), reference.context().id());
  }

  public String explainWhyNotSatisfied() {
    return isConditionSatisfied() ? null : activationCondition.explainUnsatisfied();
  }

}
