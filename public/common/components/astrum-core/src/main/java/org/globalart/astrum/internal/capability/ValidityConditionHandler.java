
package org.globalart.astrum.internal.capability;

import org.globalart.astrum.capability.CapabilityContextAware;
import org.globalart.astrum.capability.CapabilityRegistry;
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
 * Handles capability automatic removing by reacting to capability validity condition being satisfied/unsatisfied.
 *
 * @since capabilities 2.0
 */
public class ValidityConditionHandler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final EventManager eventManager;

  private final DefaultCapabilityReference reference;

  private final CapabilityRegistry capabilityRegistry;

  private final Conditions conditions;

  private Condition astrumActiveCondition;

  private Condition validityCondition;

  @Autowired
  public ValidityConditionHandler(
      final EventManager eventManager,
      final CapabilityRegistry capabilityRegistry,
      final Conditions conditions,
      final DefaultCapabilityReference reference)
  {
    this.eventManager = checkNotNull(eventManager);
    this.capabilityRegistry = checkNotNull(capabilityRegistry);
    this.conditions = checkNotNull(conditions);
    this.reference = checkNotNull(reference);
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final ConditionEvent.Satisfied event) {
    if (event.getCondition() == astrumActiveCondition) {
      bindValidity();
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final ConditionEvent.Unsatisfied event) {
    if (event.getCondition() == astrumActiveCondition) {
      releaseValidity();
    }
    else if (event.getCondition() == validityCondition) {
      reference.disable();
      try {
        capabilityRegistry.remove(reference.context().id());
      }
      catch (Exception e) {
        log.error("Failed to remove capability with id '{}'", reference.context().id(), e);
      }
    }
  }

  ValidityConditionHandler bind() {
    if (astrumActiveCondition == null) {
      astrumActiveCondition = conditions.astrum().active();
      astrumActiveCondition.bind();
      eventManager.register(this);
      if (astrumActiveCondition.isSatisfied()) {
        handle(new ConditionEvent.Satisfied(astrumActiveCondition));
      }
    }
    return this;
  }

  ValidityConditionHandler release() {
    if (astrumActiveCondition != null) {
      handle(new ConditionEvent.Unsatisfied(astrumActiveCondition));
      eventManager.unregister(this);
      astrumActiveCondition.release();
    }
    return this;
  }

  private ValidityConditionHandler bindValidity() {
    if (validityCondition == null) {
      try {
        validityCondition = reference.capability().validityCondition();
        if (validityCondition instanceof CapabilityContextAware) {
          ((CapabilityContextAware) validityCondition).setContext(reference.context());
        }
      }
      catch (Exception e) {
        validityCondition = conditions.always(
            "Always satisfied (failed to determine validity condition)");
        log.error(
            "Could not get validation condition from capability {} ({}). Considering it as always valid",
            new Object[]{reference.capability(), reference.context().id(), e});
      }
      if (validityCondition == null) {
        validityCondition = conditions.always("Always satisfied (capability has no validity condition)");
      }
      validityCondition.bind();
    }
    return this;
  }

  private ValidityConditionHandler releaseValidity() {
    if (validityCondition != null) {
      validityCondition.release();
      validityCondition = null;
    }
    return this;
  }

  @Override
  public String toString() {
    String condition = astrumActiveCondition.toString();
    if (validityCondition != null) {
      condition = validityCondition + " WHEN " + condition;
    }
    return String.format(
        "Watching '%s' condition to validate/invalidate capability '%s (id=%s)'",
        condition, reference.capability(), reference.context().id());
  }

}
