
package org.globalart.astrum.repository.search.sql.store.upgrade;

import java.sql.Connection;

import org.globalart.astrum.repository.search.sql.store.upgrade.task.CleanOrphanedSearchRecordsTask;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.doReturn;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.spy;
import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class RemoveOrphanedSearchRecordsMigrationStep_2_76Test
{
  @Mock
  private Connection connection;

  @Mock
  private UpgradeTaskScheduler upgradeTaskScheduler;

  private RemoveOrphanedSearchRecordsMigrationStep_2_76 underTest;

  @BeforeEach
  void setUp() {
    underTest = spy(new RemoveOrphanedSearchRecordsMigrationStep_2_76(upgradeTaskScheduler));
  }

  @Test
  void testSchedulesTaskIfTablesExist() throws Exception {
    doReturn(true).when(underTest).tableExists(eq(connection), anyString());

    underTest.migrate(connection);

    verify(underTest).tableExists(eq(connection), eq("search_components"));
    verify(underTest).tableExists(eq(connection), eq("search_assets"));
    verify(upgradeTaskScheduler).schedule(
        upgradeTaskScheduler.createTaskConfigurationInstance(CleanOrphanedSearchRecordsTask.TYPE_ID));
  }

  @Test
  void testDoesNotScheduleTaskIfTablesDoNotExist() throws Exception {
    doReturn(false).when(underTest).tableExists(eq(connection), anyString());

    underTest.migrate(connection);

    verify(underTest).tableExists(eq(connection), eq("search_components"));
    verify(underTest).tableExists(eq(connection), eq("search_assets"));
    verify(upgradeTaskScheduler, never()).schedule(
        upgradeTaskScheduler.createTaskConfigurationInstance(CleanOrphanedSearchRecordsTask.TYPE_ID));
  }
}
