
package org.globalart.astrum.security.config;

import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReadWriteLock;
import java.util.concurrent.locks.ReentrantReadWriteLock;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.concurrent.Locks;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.internal.SecurityContributionChangedEvent;

import com.google.common.base.Preconditions;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Preconditions.checkState;

/**
 * Mutable {@link SecurityContributor}.
 *
 * @since 3.1
 */
public class MutableSecurityContributor
    implements SecurityContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final SecurityConfiguration model = new MemorySecurityConfiguration();

  private final ReadWriteLock readWriteLock = new ReentrantReadWriteLock();

  private boolean initialized;

  @Nullable
  private EventManager eventManager;

  @Nullable
  private SecurityConfigurationManager configurationManager;

  @Autowired
  protected void initialize(final EventManager eventManager, final SecurityConfigurationManager configurationManager) {
    checkState(!initialized, "already initialized");
    this.eventManager = Preconditions.checkNotNull(eventManager);
    this.configurationManager = Preconditions.checkNotNull(configurationManager);
    initial(model);
    initialized = true;
  }

  /**
   * Initial security contribution.
   *
   * @since 3.1
   */
  protected void initial(final SecurityConfiguration model) {
    // defaults to no initial contribution
  }

  @Override
  public SecurityConfiguration getContribution() {
    checkState(initialized, "not initialized");
    Lock lock = Locks.read(readWriteLock);
    try {
      return model;
    }
    finally {
      lock.unlock();
    }
  }

  /**
   * Exposes ability to mutate {@link SecurityConfiguration}.
   */
  public interface Mutator
  {
    void apply(SecurityConfiguration model, SecurityConfigurationManager configurationManager);
  }

  public void apply(final Mutator mutator) {
    checkState(initialized, "not initialized");
    checkNotNull(mutator);

    Lock lock = Locks.write(readWriteLock);
    try {
      mutator.apply(model, configurationManager);
    }
    finally {
      lock.unlock();
    }

    eventManager.post(new SecurityContributionChangedEvent());
  }

  /**
   * @since 3.16
   */
  protected void maybeAddPrivilege(final SecurityConfiguration model, final CPrivilege privilege) {
    if (model.getPrivilege(privilege.getId()) == null) {
      model.addPrivilege(privilege);
    }
  }
}
