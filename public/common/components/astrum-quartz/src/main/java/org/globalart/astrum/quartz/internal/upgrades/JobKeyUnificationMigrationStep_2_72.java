
package org.globalart.astrum.quartz.internal.upgrades;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Schedules {@link QuartzJobKeyUnificationTask} to unify Task Configuration ID with Quartz Job Key for optimal task
 * lookup
 * performance.
 */
@Component
public class JobKeyUnificationMigrationStep_2_72
    implements DatabaseMigrationStep
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeTaskScheduler taskScheduler;

  @Autowired
  public JobKeyUnificationMigrationStep_2_72(final UpgradeTaskScheduler taskScheduler) {
    this.taskScheduler = checkNotNull(taskScheduler);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.72");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    taskScheduler
        .schedule(taskScheduler.createTaskConfigurationInstance(QuartzJobKeyUnificationTaskDescriptor.TYPE_ID));
  }
}
