
package org.globalart.astrum.repository.content.store.internal;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryStartedEvent;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.events.TaskDeletedEvent;
import org.globalart.astrum.scheduling.schedule.ScheduleFactoryImpl;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.InOrder;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.stream.Collectors.toList;
import static org.hamcrest.Matchers.allOf;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentCaptor.forClass;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.inOrder;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.config.ConfigurationConstants.DATA_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.CONTENT_STORE_FIELD_ID;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.FORMAT_FIELD_ID;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.TYPE_ID;

/**
 * Test {@link AssetBlobCleanupTaskManager}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class AssetBlobCleanupTaskManagerTest
{
  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private Format rawFormat;

  @Mock
  private Format mavenFormat;

  @Mock
  private Repository rawRepository;

  @Mock
  private Repository mavenReleaseRepository;

  @Mock
  private Repository mavenSnapshotRepository;

  @Mock
  private Configuration rawConfiguration;

  @Mock
  private Configuration mavenReleaseConfiguration;

  @Mock
  private Configuration mavenSnapshotConfiguration;

  @Mock
  private TaskInfo mavenReleaseTaskInfo;

  @Before
  public void setUp() {
    when(rawFormat.getValue()).thenReturn("raw");
    when(mavenFormat.getValue()).thenReturn("maven");

    when(rawRepository.getFormat()).thenReturn(rawFormat);
    when(mavenReleaseRepository.getFormat()).thenReturn(mavenFormat);
    when(mavenSnapshotRepository.getFormat()).thenReturn(mavenFormat);

    when(rawConfiguration.attributes(STORAGE))
        .thenReturn(new NestedAttributesMap(STORAGE, ImmutableMap.of(DATA_STORE_NAME, "rawContentStore")));
    when(mavenReleaseConfiguration.attributes(STORAGE))
        .thenReturn(new NestedAttributesMap(STORAGE, ImmutableMap.of(DATA_STORE_NAME, "mavenReleaseStore")));
    when(mavenSnapshotConfiguration.attributes(STORAGE))
        .thenReturn(new NestedAttributesMap(STORAGE, ImmutableMap.of(DATA_STORE_NAME, "mavenSnapshotStore")));

    when(rawRepository.getConfiguration()).thenReturn(rawConfiguration);
    when(mavenReleaseRepository.getConfiguration()).thenReturn(mavenReleaseConfiguration);
    when(mavenSnapshotRepository.getConfiguration()).thenReturn(mavenSnapshotConfiguration);

    when(taskScheduler.createTaskConfigurationInstance(TYPE_ID)).thenAnswer(call -> new TaskConfiguration());

    when(taskScheduler.getScheduleFactory()).thenReturn(new ScheduleFactoryImpl());

    TaskConfiguration mavenReleaseTaskConfiguration = new TaskConfiguration();
    mavenReleaseTaskConfiguration.setString(FORMAT_FIELD_ID, "maven");
    mavenReleaseTaskConfiguration.setString(CONTENT_STORE_FIELD_ID, "mavenReleaseStore");
    when(mavenReleaseTaskInfo.getConfiguration()).thenReturn(mavenReleaseTaskConfiguration);
    when(mavenReleaseTaskInfo.getTypeId()).thenReturn(TYPE_ID);
  }

  @Test
  public void testAutomaticScheduling() throws Exception {
    AssetBlobCleanupTaskManager taskManager = new AssetBlobCleanupTaskManager(taskScheduler);
    ArgumentCaptor<TaskConfiguration> configCaptor = forClass(TaskConfiguration.class);
    InOrder inOrder = inOrder(taskScheduler);

    taskManager.on(new RepositoryStartedEvent(rawRepository));
    taskManager.on(new RepositoryStartedEvent(mavenReleaseRepository));

    inOrder.verify(taskScheduler, never()).scheduleTask(any(), any());

    // starting manager should schedule one task per format+store combo

    taskManager.start();

    inOrder.verify(taskScheduler).scheduleTask(configCaptor.capture(), any());
    inOrder.verify(taskScheduler).scheduleTask(configCaptor.capture(), any());
    assertThat(configCaptor.getAllValues().stream().map(TaskConfiguration::asMap).collect(toList()),
        containsInAnyOrder(
            allOf(hasEntry(FORMAT_FIELD_ID, "raw"), hasEntry(CONTENT_STORE_FIELD_ID, "rawContentStore")),
            allOf(hasEntry(FORMAT_FIELD_ID, "maven"), hasEntry(CONTENT_STORE_FIELD_ID, "mavenReleaseStore"))));

    // creating a repository using a new format+store combo after manager has started should schedule a new task

    taskManager.on(new RepositoryStartedEvent(mavenSnapshotRepository));

    inOrder.verify(taskScheduler).scheduleTask(configCaptor.capture(), any());
    assertThat(configCaptor.getValue().asMap(),
        allOf(hasEntry(FORMAT_FIELD_ID, "maven"), hasEntry(CONTENT_STORE_FIELD_ID, "mavenSnapshotStore")));

    // creating a repository using a known format+store combo won't schedule a new task

    taskManager.on(new RepositoryStartedEvent(mavenReleaseRepository));

    inOrder.verify(taskScheduler, never()).scheduleTask(any(), any());

    // explicitly deleting a task should make the manager forget about that format+store combo...

    taskManager.on(new TaskDeletedEvent(mavenReleaseTaskInfo));

    // ...so a newly created repository using that format+store combo will again schedule a new task

    taskManager.on(new RepositoryStartedEvent(mavenReleaseRepository));

    inOrder.verify(taskScheduler).scheduleTask(configCaptor.capture(), any());
    assertThat(configCaptor.getValue().asMap(),
        allOf(hasEntry(FORMAT_FIELD_ID, "maven"), hasEntry(CONTENT_STORE_FIELD_ID, "mavenReleaseStore")));

    taskManager.stop();

    inOrder.verifyNoMoreInteractions();
  }
}
