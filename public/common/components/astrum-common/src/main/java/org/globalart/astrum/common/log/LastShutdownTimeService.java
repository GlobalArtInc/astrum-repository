
package org.globalart.astrum.common.log;

import java.util.Date;
import java.util.Optional;

/**
 * Service which provides an estimate of when astrum was last shutdown.
 *
 * @since 3.13
 */
public interface LastShutdownTimeService
{
  Optional<Date> estimateLastShutdownTime();
}
