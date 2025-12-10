
package org.globalart.astrum.internal.security.apikey.upgrade;

import java.sql.Connection;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.upgrade.datastore.DefinedUpgradeRound;
import org.globalart.astrum.upgrade.datastore.RepeatableDatabaseMigrationStep;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * An upgrade step which triggers the task to move from {@code api_key} to {@code api_key_v2}
 */
@Component
public class ApiKeySecretsDatabaseMigrationStep
    implements RepeatableDatabaseMigrationStep, DefinedUpgradeRound
{
  private final UpgradeTaskScheduler scheduler;

  @Autowired
  public ApiKeySecretsDatabaseMigrationStep(final UpgradeTaskScheduler scheduler) {
    this.scheduler = checkNotNull(scheduler);
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    scheduler.schedule(scheduler.createTaskConfigurationInstance(ApiKeyToSecretsTask.TYPE_ID));
  }

  @Override
  public Integer getChecksum() {
    return 1;
  }

  @Override
  public int getUpgradeRound() {
    // We need to be run after UserTokenLdapSamlDualRealmUserTokensMigrationStep
    return 2;
  }
}
