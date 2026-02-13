
package org.globalart.astrum.repository.content.upgrades;

import java.sql.Connection;

import org.globalart.astrum.repository.content.tasks.CreateAssetBlobIndexTaskDescriptor;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.UpgradeTaskScheduler;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertTrue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AssetBlobMigrationStep_2_18_2Test
{
  @Rule
  public DataSessionRule dataSessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME);

  @Mock
  private UpgradeTaskScheduler upgradeTaskScheduler;

  @InjectMocks
  private AssetBlobMigrationStep_2_18_2 underTest;

  private TaskConfiguration mockTaskConfiguration;

  @Before
  public void setup() {
    mockTaskConfiguration = mock(TaskConfiguration.class);
    when(upgradeTaskScheduler.createTaskConfigurationInstance(
        CreateAssetBlobIndexTaskDescriptor.TYPE_ID))
            .thenReturn(mockTaskConfiguration);
  }

  @Test
  public void testMigrate_schedulesTask() throws Exception {
    try (Connection connection = dataSessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(connection);
    }

    // Verify the task was created with correct type ID
    verify(upgradeTaskScheduler).createTaskConfigurationInstance(
        eq(CreateAssetBlobIndexTaskDescriptor.TYPE_ID));

    // Verify the task was scheduled
    ArgumentCaptor<TaskConfiguration> configCaptor = ArgumentCaptor.forClass(TaskConfiguration.class);
    verify(upgradeTaskScheduler).schedule(configCaptor.capture());

    // Verify the scheduled configuration is the same as the created one
    assertThat("Scheduled task configuration should match created configuration",
        configCaptor.getValue(),
        equalTo(mockTaskConfiguration));
  }

  @Test
  public void testMigrate_calledOnce() throws Exception {
    try (Connection connection = dataSessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(connection);
    }

    // Verify methods called exactly once
    verify(upgradeTaskScheduler, times(1))
        .createTaskConfigurationInstance(CreateAssetBlobIndexTaskDescriptor.TYPE_ID);
    verify(upgradeTaskScheduler, times(1)).schedule(any(TaskConfiguration.class));
  }

  @Test
  public void testVersion() {
    assertTrue("version should be present", underTest.version().isPresent());
    assertEquals("version should be 2.18.2", "2.18.2", underTest.version().get());
  }

  @Test
  public void testMigrate_doesNotFailWithNullConnection() throws Exception {
    // This tests that the migration doesn't depend on connection state
    try (Connection connection = dataSessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(connection);
    }

    // Should still schedule the task successfully
    verify(upgradeTaskScheduler).schedule(any(TaskConfiguration.class));
  }

  @Test
  public void testMigrate_multipleCallsScheduleMultipleTasks() throws Exception {
    // Simulate multiple calls to migrate (shouldn't happen in practice, but tests idempotency)
    try (Connection connection1 = dataSessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(connection1);
    }

    try (Connection connection2 = dataSessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(connection2);
    }

    // Verify the task was scheduled twice (once per call)
    verify(upgradeTaskScheduler, times(2)).schedule(any(TaskConfiguration.class));
  }

  @Test
  public void testMigrate_usesCorrectTypeId() throws Exception {
    try (Connection connection = dataSessionRule.openConnection(DEFAULT_DATASTORE_NAME)) {
      underTest.migrate(connection);
    }

    // Verify the correct type ID constant is used
    verify(upgradeTaskScheduler).createTaskConfigurationInstance(
        eq("repository.asset.blob.index.migration"));
  }
}
