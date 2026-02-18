
package org.globalart.astrum.repository.capability.internal;

import java.util.concurrent.locks.ReentrantReadWriteLock;
import java.util.function.Supplier;

import org.globalart.astrum.capability.CapabilityContext;
import org.globalart.astrum.capability.CapabilityContextAware;
import org.globalart.astrum.capability.CapabilityEvent;
import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.capability.condition.ConditionSupport;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryCreatedEvent;
import org.globalart.astrum.repository.manager.RepositoryLoadedEvent;
import org.globalart.astrum.repository.manager.RepositoryManager;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * Support class for repository conditions.
 *
 * @since 3.0
 */
public abstract class RepositoryConditionSupport
    extends ConditionSupport
    implements CapabilityContextAware
{

  private final RepositoryManager repositoryManager;

  private final Supplier<String> repositoryName;

  private final ReentrantReadWriteLock bindLock;

  private CapabilityIdentity capabilityIdentity;

  private String repositoryBeforeLastUpdate;

  public RepositoryConditionSupport(
      final EventManager eventManager,
      final RepositoryManager repositoryManager,
      final Supplier<String> repositoryName)
  {
    super(eventManager, false);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.repositoryName = checkNotNull(repositoryName);
    bindLock = new ReentrantReadWriteLock();
  }

  @Override
  public RepositoryConditionSupport setContext(final CapabilityContext context) {
    checkState(!isActive(), "Cannot contextualize when already bounded");
    checkState(capabilityIdentity == null, "Already contextualized with id '" + capabilityIdentity + "'");
    capabilityIdentity = context.id();

    return this;
  }

  @Override
  protected void doBind() {
    try {
      bindLock.writeLock().lock();
      for (final Repository repository : repositoryManager.browse()) {
        handle(new RepositoryCreatedEvent(repository));
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

  public abstract void handle(final RepositoryCreatedEvent event);

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

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final RepositoryLoadedEvent event) {
    handle(new RepositoryCreatedEvent(event.getRepository()));
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.BeforeUpdate event) {
    if (event.getReference().context().id().equals(capabilityIdentity)) {
      repositoryBeforeLastUpdate = getRepositoryName();
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void handle(final CapabilityEvent.AfterUpdate event) {
    if (event.getReference().context().id().equals(capabilityIdentity)) {
      if (!sameRepositoryAs(repositoryBeforeLastUpdate)) {
        try {
          bindLock.writeLock().lock();
          for (final Repository repository : repositoryManager.browse()) {
            handle(new RepositoryCreatedEvent(repository));
          }
        }
        finally {
          bindLock.writeLock().unlock();
        }
      }
    }
  }

  /**
   * Checks that condition is about the passed in repository name.
   *
   * @param repositoryName to check
   * @return true, if condition repository matches the specified repository name
   */
  protected boolean sameRepositoryAs(final String repositoryName) {
    return repositoryName != null && repositoryName.equals(getRepositoryName());
  }

  protected String getRepositoryName() {
    return repositoryName.get();
  }

}
