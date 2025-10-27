
package org.globalart.astrum.common.event;

import java.util.function.Supplier;

import static com.google.common.base.Preconditions.checkState;
import static java.lang.Boolean.TRUE;

/**
 * Event helpers.
 *
 * @since 3.2
 */
public class EventHelper
{
  private static final ThreadLocal<Boolean> isReplicating = new ThreadLocal<>();

  private EventHelper() {
    // empty
  }

  /**
   * Is this thread currently replicating remote events from another node?
   */
  public static boolean isReplicating() {
    return TRUE.equals(isReplicating.get());
  }

  /**
   * Calls the given {@link Supplier} while flagged as replicating remote events.
   */
  public static <T> T asReplicating(final Supplier<T> supplier) {
    checkState(!isReplicating(), "Replication already in progress");
    isReplicating.set(TRUE);
    try {
      return supplier.get();
    }
    finally {
      isReplicating.remove();
    }
  }

  /**
   * Calls the given {@link Runnable} while flagged as replicating remote events.
   */
  public static void asReplicating(final Runnable runnable) {
    checkState(!isReplicating(), "Replication already in progress");
    isReplicating.set(TRUE);
    try {
      runnable.run();
    }
    finally {
      isReplicating.remove();
    }
  }
}
