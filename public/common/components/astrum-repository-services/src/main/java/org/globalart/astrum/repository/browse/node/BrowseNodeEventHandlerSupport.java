
package org.globalart.astrum.repository.browse.node;

import java.util.concurrent.atomic.AtomicBoolean;

import org.globalart.astrum.common.lifecycle.LifecycleSupport;

/**
 * Manages common tasks for the BrowseNodeEventHandler
 * 
 * @since 3.35
 */
public abstract class BrowseNodeEventHandlerSupport
    extends LifecycleSupport
    implements BrowseNodeEventHandler
{
  private final AtomicBoolean pauseHandling = new AtomicBoolean(false);

  @Override
  public boolean shouldHandle() {
    return !pauseHandling.get();
  }

  @Override
  public void pauseEventProcessing() {
    log.info("BrowseNode event processing has been paused");
    pauseHandling.set(true);
  }

  @Override
  public void resumeEventProcessing() {
    log.info("BrowseNode event processing has been resumed");
    pauseHandling.set(false);
  }
}
