
package org.globalart.astrum.common.cooperation2;

import java.time.Duration;

import org.globalart.astrum.common.io.Cooperation;

/**
 * Configuration holder for {@link Cooperation} points.
 */
public class Config
{
  protected int majorTimeoutSeconds = 0;

  protected int minorTimeoutSeconds = 0;

  protected int threadsPerKey = 0;

  public Duration majorTimeout() {
    return Duration.ofSeconds(majorTimeoutSeconds);
  }

  public Duration minorTimeout() {
    return Duration.ofSeconds(minorTimeoutSeconds);
  }

  public int threadsPerKey() {
    return threadsPerKey;
  }

  protected Config copy() {
    Config copy = new Config();
    copy.majorTimeoutSeconds = majorTimeoutSeconds;
    copy.minorTimeoutSeconds = minorTimeoutSeconds;
    copy.threadsPerKey = threadsPerKey;
    return copy;
  }
}
