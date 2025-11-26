
package org.globalart.astrum.transaction;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicIntegerArray;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.System.currentTimeMillis;
import static java.util.concurrent.TimeUnit.MILLISECONDS;
import static java.util.stream.IntStream.range;
import static java.util.stream.LongStream.rangeClosed;

/**
 * Maintains a rolling window of counts for the given time period.
 *
 * @since 3.16
 */
class RollingStats
{
  private final AtomicIntegerArray buckets;

  private final TimeUnit tickUnit;

  private volatile long tick;

  public RollingStats(final int size, final TimeUnit tickUnit) {
    this.buckets = new AtomicIntegerArray(size);
    this.tickUnit = checkNotNull(tickUnit);
    this.tick = currentTick();
  }

  /**
   * Adds a count to the window at the latest tick.
   */
  public void mark() {
    maybeShiftWindow();
    buckets.getAndIncrement(index(tick));
  }

  /**
   * Returns a sum of all counts in the window.
   */
  public int sum() {
    maybeShiftWindow();
    return range(0, buckets.length()).map(buckets::get).sum();
  }

  /**
   * Returns the current tick based on the system clock.
   */
  private long currentTick() {
    return tickUnit.convert(currentTimeMillis(), MILLISECONDS);
  }

  /**
   * Converts the given tick to an index into the window.
   */
  private int index(final long _tick) {
    return (int) (_tick % buckets.length());
  }

  /**
   * Compares the last tick to the current tick and shifts the window accordingly, zeroing out stale elements.
   */
  private void maybeShiftWindow() {
    long newTick = currentTick();
    if (tick < newTick) {
      synchronized (this) {
        if (tick < newTick) {
          // move round circular window: any elements after the last tick, up to and including current tick, is now
          // stale
          rangeClosed(tick + 1, newTick).limit(buckets.length()).mapToInt(this::index).forEach(i -> buckets.set(i, 0));
          tick = newTick;
        }
      }
    }
  }
}
