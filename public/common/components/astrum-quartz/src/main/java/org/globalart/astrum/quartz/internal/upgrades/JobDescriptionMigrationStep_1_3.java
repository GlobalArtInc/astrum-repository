
package org.globalart.astrum.quartz.internal.upgrades;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Resolve issue when job can't be scheduled due to limited size of 'description' column
 *
 * @since 3.37
 */
@Component
public class JobDescriptionMigrationStep_1_3
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String QRTZ_JOB_DETAILS = "qrtz_job_details";

  private static final String QUARTZ_TRIGGERS = "qrtz_triggers";

  private static final String STATEMENT = "ALTER TABLE %s ALTER COLUMN description TYPE text";

  @Override
  public Optional<String> version() {
    return Optional.of("1.3");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    if (tableExists(connection, QRTZ_JOB_DETAILS) && tableExists(connection, QUARTZ_TRIGGERS)) {
      log.info("Changing 'description' column type from varchar(250) to text in table {}", QRTZ_JOB_DETAILS);
      runStatement(connection, String.format(STATEMENT, QRTZ_JOB_DETAILS));

      log.info("Changing 'description' column type from varchar(250) to text in table {}", QUARTZ_TRIGGERS);
      runStatement(connection, String.format(STATEMENT, QUARTZ_TRIGGERS));
    }
    else {
      log.warn("Table {} or {} doesn't exist, upgrade step {} will be skipped",
          QRTZ_JOB_DETAILS, QUARTZ_TRIGGERS, this.getClass().getSimpleName());
    }
  }
}
