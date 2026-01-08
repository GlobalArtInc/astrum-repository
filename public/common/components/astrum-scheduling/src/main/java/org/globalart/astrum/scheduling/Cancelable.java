
package org.globalart.astrum.scheduling;

/**
 * Cancelable interface, that should be implemented by tasks supports cancellation.
 *
 * @since 3.0
 */
public interface Cancelable
{
  /**
   * Cancels the task.
   */
  void cancel();

  /**
   * Returns {@code true} if canceled.
   */
  boolean isCanceled();
}
