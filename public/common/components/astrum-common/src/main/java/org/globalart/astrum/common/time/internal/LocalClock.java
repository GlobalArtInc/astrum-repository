
package org.globalart.astrum.common.time.internal;

import java.time.OffsetDateTime;

import org.globalart.astrum.common.time.Clock;
import org.springframework.stereotype.Component;

/**
 * A provider of the current time.
 *
 * @since 3.0
 */
@Component
public class LocalClock
    implements Clock
{
  @Override
  public OffsetDateTime clusterTime() {
    return OffsetDateTime.now();
  }
}
