
package org.globalart.astrum.scheduling;

import java.util.concurrent.atomic.AtomicBoolean;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Support for per-thread storing of cancelable flag.
 *
 * Periodically checking the {@link #checkCancellation()} is the preferred way to detect cancellation in components
 * outside the tasks. Within task, you have the {@link TaskSupport#isCanceled()} method.
 *
 * @since 3.0
 */
public class CancelableHelper
{
  private CancelableHelper() {
    // empty
  }

  // FIXME: its not clear why we have to install/remove a flag here, vs managing this per-thread.

  private static final ThreadLocal<AtomicBoolean> currentFlagHolder = new ThreadLocal<>();

  public static void set(final AtomicBoolean flag) {
    checkNotNull(flag);
    currentFlagHolder.set(flag);
  }

  public static void remove() {
    currentFlagHolder.remove();
  }

  /**
   * Throws {@link TaskInterruptedException} if current task is canceled or interrupted.
   */
  public static boolean checkCancellation() {
    Thread.yield();
    AtomicBoolean current = currentFlagHolder.get();
    if (current != null && current.get()) {
      throw new TaskInterruptedException("Thread '" + Thread.currentThread().getName() + "' is canceled", true);
    }
    if (Thread.interrupted()) {
      throw new TaskInterruptedException("Thread '" + Thread.currentThread().getName() + "' is interrupted", false);
    }
    return true;
  }
}
