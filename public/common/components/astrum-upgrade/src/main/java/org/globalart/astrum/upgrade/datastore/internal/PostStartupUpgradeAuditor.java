
package org.globalart.astrum.upgrade.datastore.internal;

import java.util.Queue;
import java.util.concurrent.ConcurrentLinkedQueue;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.common.upgrade.events.UpgradeEventSupport;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import org.springframework.stereotype.Component;

/**
 * Stores auditable db upgrade events until after startup when auditing is enabled
 */
@Component
@ManagedLifecycle(phase = TASKS)
public class PostStartupUpgradeAuditor
    extends StateGuardLifecycleSupport
{
  private final Queue<UpgradeEventSupport> events = new ConcurrentLinkedQueue<>();

  private final EventManager eventManager;

  @Autowired
  public PostStartupUpgradeAuditor(final EventManager eventManager) {
    this.eventManager = checkNotNull(eventManager);
  }

  @Override
  protected void doStart() {
    events.forEach(eventManager::post);
    events.clear();
  }

  public void post(final UpgradeEventSupport event) {
    if (!isStarted()) {
      events.add(event);
    }
    else {
      // The service has started which means that the auditor should work if enabled.
      eventManager.post(event);
    }
  }
}
