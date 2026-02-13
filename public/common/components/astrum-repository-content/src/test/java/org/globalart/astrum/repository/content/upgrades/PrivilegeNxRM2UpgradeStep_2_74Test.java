
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
public class PrivilegeNxRM2UpgradeStep_2_74Test
{
  @Mock
  private UpgradeTaskScheduler upgradeTaskScheduler;

  private PrivilegeNxRM2UpgradeStep_2_74 underTest;

  @Before
  public void setup() {
    underTest = new PrivilegeNxRM2UpgradeStep_2_74(upgradeTaskScheduler);
  }

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();

    assertThat(version).isPresent();
    assertThat(version.get()).isEqualTo("2.74");
  }

  @Test
  public void testMigrate() throws Exception {
    TaskConfiguration taskConfiguration = mock(TaskConfiguration.class);
    when(upgradeTaskScheduler.createTaskConfigurationInstance("nxrm2.privilege.upgrade"))
        .thenReturn(taskConfiguration);

    underTest.migrate(mock(Connection.class));

    verify(upgradeTaskScheduler).createTaskConfigurationInstance("nxrm2.privilege.upgrade");
    verify(upgradeTaskScheduler).schedule(taskConfiguration);
  }
}
