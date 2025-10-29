
package org.globalart.astrum.common.scheduling;

import java.time.Duration;

/**
 * A service that periodically runs {@link Runnable} tasks.
 *
 * @since 3.0
 */
public interface PeriodicJobService
{
  /**
   * Starts the service if it is not already started. Each successful invocation of this method must eventually be
   * paired with a corresponding invocation of {@link #stopUsing()}.
   *
   * @since 3.1
   */
  void startUsing();

  /**
   * Signals the caller no longer needs the service. The service is stopped if this invocation balances out all previous
   * invocations of {@link #startUsing()}.
   *
   * @since 3.1
   */
  void stopUsing();

  /**
   * Runs the provided task a single time after the specified number of seconds.
   */
  void runOnce(final Runnable runnable, final int delaySeconds);

  /**
   * Add a job to be periodically executed.
   *
   * @return a {@link PeriodicJob} that must be closed when the job is no longer necessary.
   */
  PeriodicJob schedule(Runnable runnable, final int repeatPeriodSeconds);

  /**
   * Add a job to be periodically executed.
   *
   * @return a {@link PeriodicJob} that must be closed when the job is no longer necessary.
   */
  default PeriodicJob schedule(final Runnable runnable, final Duration repeatPeriod) {
    return schedule(runnable, repeatPeriod, repeatPeriod);
  }

  /**
   * Add a job to be periodically executed.
   *
   * @return a {@link PeriodicJob} that must be closed when the job is no longer necessary.
   */
  PeriodicJob schedule(Runnable runnable, final Duration delay, final Duration repeatPeriod);

  interface PeriodicJob
  {
    void cancel();
  }
}
