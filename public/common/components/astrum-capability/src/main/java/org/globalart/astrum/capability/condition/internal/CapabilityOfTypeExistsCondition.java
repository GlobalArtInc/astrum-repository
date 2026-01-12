
package org.globalart.astrum.capability.condition.internal;

import java.util.concurrent.locks.ReentrantReadWriteLock;

import org.globalart.astrum.capability.CapabilityDescriptor;
import org.globalart.astrum.capability.CapabilityDescriptorRegistry;
import org.globalart.astrum.capability.CapabilityEvent;
import org.globalart.astrum.capability.CapabilityReference;
import org.globalart.astrum.capability.CapabilityRegistry;
import org.globalart.astrum.capability.CapabilityType;
import org.globalart.astrum.capability.condition.ConditionSupport;
import org.globalart.astrum.common.event.EventManager;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * A condition that is satisfied when a capability of a specified type exists.
 *
 * @since capabilities 2.0
 */
public class CapabilityOfTypeExistsCondition
    extends ConditionSupport
{

  private final CapabilityRegistry capabilityRegistry;

  private final ReentrantReadWriteLock bindLock;

  final CapabilityType type;

  final String typeName;

  public CapabilityOfTypeExistsCondition(
      final EventManager eventManager,
      final CapabilityDescriptorRegistry descriptorRegistry,
      final CapabilityRegistry capabilityRegistry,
      final CapabilityType type)
  {
    super(eventManager);
    this.capabilityRegistry = checkNotNull(capabilityRegistry);
    this.type = checkNotNull(type);
    final CapabilityDescriptor descriptor = checkNotNull(descriptorRegistry).get(type);
    typeName = descriptor == null ? type.toString() : descriptor.name();
    bindLock = new ReentrantReadWriteLock();
  }

  @Override
  protected void doBind() {
    try {
      bindLock.writeLock().lock();
      for (final CapabilityReference reference : capabilityRegistry.getAll()) {
        handle(new CapabilityEvent.Created(reference));
      }
    }
    finally {
      bindLock.writeLock().unlock();
    }
    getEventManager().register(this);
  }

  @Override
  public void doRelease() {
    getEventManager().unregister(this);
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.Created event) {
    if (!isSatisfied() && type.equals(event.getReference().context().type())) {
      checkAllCapabilities();
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.AfterRemove event) {
    if (isSatisfied() && type.equals(event.getReference().context().type())) {
      checkAllCapabilities();
    }
  }

  void checkAllCapabilities() {
    for (final CapabilityReference ref : capabilityRegistry.getAll()) {
      if (isSatisfiedBy(ref)) {
        setSatisfied(true);
        return;
      }
    }
    setSatisfied(false);
  }

  boolean isSatisfiedBy(final CapabilityReference reference) {
    return type.equals(reference.context().type());
  }

  @Override
  protected void setSatisfied(final boolean satisfied) {
    try {
      bindLock.readLock().lock();
      super.setSatisfied(satisfied);
    }
    finally {
      bindLock.readLock().unlock();
    }
  }

  @Override
  public String toString() {
    return type + " exists";
  }

  @Override
  public String explainSatisfied() {
    return typeName + " exists";
  }

  @Override
  public String explainUnsatisfied() {
    return typeName + " does not exist";
  }

}
