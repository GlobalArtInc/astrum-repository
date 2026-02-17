
package org.globalart.astrum.repository.browse.node;

/**
 * EventHandler that manages browse nodes realtime
 * 
 * @since 3.35
 */
public interface BrowseNodeEventHandler
{
  /**
   * event handling should only be carried out if this is true
   */
  boolean shouldHandle();

  /**
   * pause processing of events, once this happens, rebuilding browse nodes manually (via the task) will be required
   * after resuming processing
   */
  void pauseEventProcessing();

  /**
   * resume processing, with the knowledge that anything added while in the paused state did not get added to browse
   */
  void resumeEventProcessing();
}
