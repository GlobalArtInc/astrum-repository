
package org.globalart.astrum.internal.security.secrets.upgrade;

import java.sql.Connection;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.internal.security.secrets.SecretsDAO;
import org.globalart.astrum.internal.security.secrets.task.SecretsMigrationTaskDescriptor;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.verify;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SecretsMigrationStep_2_2Test
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME)
      .access(SecretsDAO.class);

  @Mock
  private UpgradeTaskScheduler upgradeTaskScheduler;

  private SecretsMigrationStep_2_2 migrationStep;

  private DataSession<?> session;

  @Before
  public void setUp() {
    migrationStep = new SecretsMigrationStep_2_2(upgradeTaskScheduler);
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
  }

  @Test
  public void testMigrationSchedulesTask() throws Exception {
    try (Connection connection = sessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      migrationStep.migrate(connection);

      verify(upgradeTaskScheduler).createTaskConfigurationInstance(SecretsMigrationTaskDescriptor.TYPE_ID);
      verify(upgradeTaskScheduler).schedule(any());
    }
  }

  @After
  public void cleanup() {
    session.close();
  }
}
