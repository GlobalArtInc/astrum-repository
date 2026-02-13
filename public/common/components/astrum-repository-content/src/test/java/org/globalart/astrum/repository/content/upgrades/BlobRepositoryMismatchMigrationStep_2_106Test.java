
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;
import java.util.Optional;

import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BlobRepositoryMismatchMigrationStep_2_106Test
{
  @Mock
  private UpgradeTaskScheduler upgradeTaskScheduler;

  private BlobRepositoryMismatchMigrationStep_2_106 underTest;

  @Before
  public void setup() {
    underTest = new BlobRepositoryMismatchMigrationStep_2_106(upgradeTaskScheduler);
  }

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();

    assertThat(version).isPresent();
    assertThat(version.get()).isEqualTo("2.106");
  }

  @Test
  public void testMigrate() throws Exception {
    TaskConfiguration taskConfiguration = mock(TaskConfiguration.class);
    when(upgradeTaskScheduler.createTaskConfigurationInstance("repository.blob.mismatch.task"))
        .thenReturn(taskConfiguration);

    underTest.migrate(mock(Connection.class));

    verify(upgradeTaskScheduler).createTaskConfigurationInstance("repository.blob.mismatch.task");
    verify(taskConfiguration).setString("repositoryName", "*");
    verify(upgradeTaskScheduler).schedule(taskConfiguration, false);
  }
}
