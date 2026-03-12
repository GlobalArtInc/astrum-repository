
package org.globalart.astrum.upgrade.internal;

import java.sql.Connection;

import org.globalart.astrum.upgrade.datastore.RepeatableDatabaseMigrationStep;

import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * No-op upgrade step.
 *
 * Previously dropped distributed_events table when clustering was disabled.
 * Now that distributed_events table is always created (regardless of HA mode),
 * this step is no longer needed.
 *
 * @since 3.38
 */
@Component
public class DistributedEventsUpgrade
    implements RepeatableDatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public Integer getChecksum() {
    return 1;
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    // No-op: distributed_events table is now always present regardless of HA mode (ASTRUM-49804)
  }
}
