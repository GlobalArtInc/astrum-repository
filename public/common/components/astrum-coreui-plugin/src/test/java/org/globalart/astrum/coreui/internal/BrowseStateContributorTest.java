
package org.globalart.astrum.coreui.internal;

import java.util.Arrays;
import java.util.Collections;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.browse.node.BrowseNodeConfiguration;
import org.globalart.astrum.repository.browse.node.RebuildBrowseNodesTaskDescriptor;
import org.globalart.astrum.scheduling.CurrentState;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.TaskState;

import com.google.common.collect.Sets;
import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.RepositoryTaskSupport.ALL_REPOSITORIES;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BrowseStateContributorTest
{
  @Mock
  private BrowseNodeConfiguration browseNodeConfiguration;

  @Mock
  private TaskScheduler taskScheduler;

  private BrowseStateContributor underTest;

  @Before
  public void setup() {
    when(browseNodeConfiguration.getMaxNodes()).thenReturn(10);
    underTest = new BrowseStateContributor(browseNodeConfiguration, taskScheduler, 60);
  }

  @Test
  public void testGetState() {
    List<TaskInfo> tasks = Arrays
        .asList(createTaskInfo(RebuildBrowseNodesTaskDescriptor.TYPE_ID, TaskState.RUNNING, ALL_REPOSITORIES));
    when(taskScheduler.listsTasks()).thenReturn(tasks);

    Map<String, Object> state = underTest.getState();

    assertThat(state.size(), is(2));
    assertThat(state.get("rebuildingRepositories"), is(Collections.singleton(ALL_REPOSITORIES)));
    assertThat(state.get("browseTreeMaxNodes"), is(10));
  }

  @Test
  public void testGetState_multipleTasks() {
    List<TaskInfo> tasks = Arrays
        .asList(createTaskInfo(RebuildBrowseNodesTaskDescriptor.TYPE_ID, TaskState.RUNNING, "repo1"),
            createTaskInfo(RebuildBrowseNodesTaskDescriptor.TYPE_ID, TaskState.RUNNING, "repo2"),
            createTaskInfo(RebuildBrowseNodesTaskDescriptor.TYPE_ID, TaskState.RUNNING, "repo3"));
    when(taskScheduler.listsTasks()).thenReturn(tasks);

    Map<String, Object> state = underTest.getState();

    assertThat(state.size(), is(2));
    assertThat(state.get("rebuildingRepositories"), is(Sets.newHashSet("repo1", "repo2", "repo3")));
    assertThat(state.get("browseTreeMaxNodes"), is(10));
  }

  @Test
  public void testGetState_firstTaskAllReposOthersIgnored() {
    List<TaskInfo> tasks = Arrays
        .asList(createTaskInfo(RebuildBrowseNodesTaskDescriptor.TYPE_ID, TaskState.RUNNING, ALL_REPOSITORIES),
            createTaskInfo(RebuildBrowseNodesTaskDescriptor.TYPE_ID, TaskState.RUNNING, "repo2"),
            createTaskInfo(RebuildBrowseNodesTaskDescriptor.TYPE_ID, TaskState.RUNNING, "repo3"));
    when(taskScheduler.listsTasks()).thenReturn(tasks);

    Map<String, Object> state = underTest.getState();

    assertThat(state.size(), is(2));
    assertThat(state.get("rebuildingRepositories"), is(Collections.singleton(ALL_REPOSITORIES)));
    assertThat(state.get("browseTreeMaxNodes"), is(10));

    verifyNoInteractions(tasks.get(1));
    verifyNoInteractions(tasks.get(2));
  }

  @Test
  public void testGetState_mixedTaskTypes() {
    List<TaskInfo> tasks = Arrays
        .asList(createTaskInfo(RebuildBrowseNodesTaskDescriptor.TYPE_ID, TaskState.RUNNING, "repo1"),
            createTaskInfo("typeId", TaskState.RUNNING, "repo2"));
    when(taskScheduler.listsTasks()).thenReturn(tasks);

    Map<String, Object> state = underTest.getState();

    assertThat(state.size(), is(2));
    assertThat(state.get("rebuildingRepositories"), is(Collections.singleton("repo1")));
    assertThat(state.get("browseTreeMaxNodes"), is(10));
  }

  private TaskInfo createTaskInfo(String typeId, TaskState runState, String repositoryName) {
    CurrentState currentState = mock(CurrentState.class);
    when(currentState.getRunState()).thenReturn(runState);

    TaskConfiguration taskConfiguration = new TaskConfiguration();
    taskConfiguration.setString("repositoryName", repositoryName);

    TaskInfo taskInfo = mock(TaskInfo.class);
    when(taskInfo.getTypeId()).thenReturn(typeId);
    when(taskInfo.getCurrentState()).thenReturn(currentState);
    when(taskInfo.getConfiguration()).thenReturn(taskConfiguration);
    return taskInfo;
  }
}
