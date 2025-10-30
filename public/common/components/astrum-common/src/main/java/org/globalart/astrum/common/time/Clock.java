
package org.globalart.astrum.common.time;

import java.time.OffsetDateTime;

import org.joda.time.DateTime;

/**
 * A provider of the current time.
 *
 * @since 3.0
 */
public interface Clock
{
  /**
   * Current time in milli-seconds.
   */
  default long millis() {
    return System.currentTimeMillis();
  };

  /**
   * Current time in nano-seconds.
   */
  default long nanos() {
    return System.nanoTime();
  }

  /**
   * Current date-time.
   */
  default DateTime dateTime() {
    return new DateTime(millis());
  }

  /**
   * Returns an {@link OffsetDateTime} for the Astrum cluster.
   */
  OffsetDateTime clusterTime();
}
