
package org.globalart.astrum.scheduling;

/**
 * Runtime exception thrown in cases when task is interrupted.
 *
 * Semantically meaning is almost same as {@link InterruptedException} except
 * this one is unchecked exception and may carry some cause.
 *
 * @since 3.0
 */
public class TaskInterruptedException
    extends RuntimeException
{
  private final boolean canceled;

  public TaskInterruptedException(final String message, final boolean canceled) {
    super(message);
    this.canceled = canceled;
  }

  public boolean isCanceled() {
    return canceled;
  }
}
