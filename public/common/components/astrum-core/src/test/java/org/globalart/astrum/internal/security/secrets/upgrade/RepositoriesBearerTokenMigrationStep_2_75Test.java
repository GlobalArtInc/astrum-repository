
package org.globalart.astrum.internal.security.secrets.upgrade;

import java.sql.Connection;

import org.globalart.astrum.internal.security.secrets.SecretsDAO;
import org.globalart.astrum.internal.security.secrets.task.RepositoriesBearerTokenConfigMigrationTaskDescriptor;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.testdb.DataSessionConfiguration;
import org.globalart.astrum.testdb.DatabaseTest;
import org.globalart.astrum.testdb.TestDataSessionSupplier;

import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.verify;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@ExtendWith(MockitoExtension.class)
class RepositoriesBearerTokenMigrationStep_2_75Test
{
  @DataSessionConfiguration(daos = SecretsDAO.class)
  TestDataSessionSupplier dataSession;

  @Mock
  private UpgradeTaskScheduler upgradeTaskScheduler;

  @InjectMocks
  private RepositoriesBearerTokenMigrationStep_2_75 migrationStep;

  @DatabaseTest
  void testMigrationSchedulesTaskWhenSecretsTableExists() throws Exception {
    try (Connection connection = dataSession.openConnection(DEFAULT_DATASTORE_NAME)) {
      migrationStep.migrate(connection);

      verify(upgradeTaskScheduler)
          .createTaskConfigurationInstance(RepositoriesBearerTokenConfigMigrationTaskDescriptor.TYPE_ID);
      verify(upgradeTaskScheduler).schedule(any());
    }
  }
}
