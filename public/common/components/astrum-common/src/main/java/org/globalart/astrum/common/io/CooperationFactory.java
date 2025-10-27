
package org.globalart.astrum.common.io;

import java.time.Duration;

import org.globalart.astrum.common.time.Time;

/**
 * Supplies {@link Cooperation} points. Not intended for use with SQL/Datastore.
 *
 * @since 3.14
 */
public interface CooperationFactory
{
  /**
   * Start configuring a new {@link Cooperation} point.
   */
  Builder configure();

  /**
   * Fluent builder for configuring {@link Cooperation} points.
   */
  interface Builder
  {
    /**
     * @param majorTimeout when waiting for the main I/O request
     */
    Builder majorTimeout(Duration majorTimeout);

    /**
     * @param majorTimeout when waiting for the main I/O request
     * @deprecated use the API utilizing java.time.Duration
     */
    @Deprecated
    default Builder majorTimeout(final Time majorTimeout) {
      return majorTimeout(Duration.ofSeconds(majorTimeout.toSeconds()));
    }

    /**
     * @param minorTimeout when waiting for any I/O dependencies
     */
    Builder minorTimeout(Duration minorTimeout);

    /**
     * @param minorTimeout when waiting for any I/O dependencies
     * @deprecated use the API utilizing java.time.Duration
     */
    @Deprecated
    default Builder minorTimeout(final Time minorTimeout) {
      return majorTimeout(Duration.ofSeconds(minorTimeout.toSeconds()));
    }

    /**
     * @param threadsPerKey limits the threads waiting under each key
     */
    Builder threadsPerKey(int threadsPerKey);

    /**
     * Builds a new {@link Cooperation} point with this configuration.
     *
     * @param id unique identifier for this cooperation point
     */
    Cooperation build(String id);
  }
}
