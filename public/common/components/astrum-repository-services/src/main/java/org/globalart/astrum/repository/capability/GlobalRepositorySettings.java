
package org.globalart.astrum.repository.capability;

import java.time.Duration;

import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Global repository settings.
 *
 * @since 3.24
 */
@Component
public class GlobalRepositorySettings
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final Duration DEFAULT_LAST_DOWNLOADED_INTERVAL = Duration.ofHours(12);

  private Duration lastDownloadedInterval = DEFAULT_LAST_DOWNLOADED_INTERVAL;

  public void setLastDownloadedInterval(final Duration lastDownloadedInterval) {
    if (lastDownloadedInterval.toHours() < 1) {
      log.warn("A lastDownloaded interval of {} seconds has been configured, a value less than"
          + " 1 hour is not recommended for performance reasons", lastDownloadedInterval.getSeconds());
    }
    this.lastDownloadedInterval = lastDownloadedInterval;
  }

  public Duration getLastDownloadedInterval() {
    return lastDownloadedInterval;
  }
}
