
package org.globalart.astrum.common.concurrent;

import java.util.concurrent.TimeUnit;
import java.util.concurrent.locks.Lock;
import java.util.concurrent.locks.ReadWriteLock;

import com.google.common.base.Throwables;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Lock helpers.
 *
 */
public class Locks
{
  private Locks() {
    // empty
  }

  /**
   * Returns locked lock.
   *
   * Uses {@link Lock#tryLock} with timeout of 60 seconds to avoid potential deadlocks.
   */
  public static Lock lock(final Lock lock) {
    checkNotNull(lock);
    try {
      if (!lock.tryLock(60, TimeUnit.SECONDS)) {
        throw new RuntimeException("Failed to obtain lock after 60 seconds");
      }
    }
    catch (InterruptedException e) {
      throw propagate(e);
    }
    return lock;
  }

  /**
   * Returns locked read-lock.
   */
  public static Lock read(final ReadWriteLock readWriteLock) {
    checkNotNull(readWriteLock);
    return lock(readWriteLock.readLock());
  }

  /**
   * Returns locked write-lock.
   */
  public static Lock write(final ReadWriteLock readWriteLock) {
    checkNotNull(readWriteLock);
    return lock(readWriteLock.writeLock());
  }

  private static RuntimeException propagate(Throwable throwable) {
    Throwables.throwIfUnchecked(throwable);
    throw new RuntimeException(throwable);
  }
}
