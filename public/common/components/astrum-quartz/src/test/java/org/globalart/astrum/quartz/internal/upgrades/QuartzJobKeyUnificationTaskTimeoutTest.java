
package org.globalart.astrum.quartz.internal.upgrades;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.TimeUnit;

import org.globalart.astrum.node.datastore.NodeHeartbeat;
import org.globalart.astrum.node.datastore.NodeHeartbeatManager;
import org.globalart.astrum.quartz.SleeperTask;
import org.globalart.astrum.quartz.internal.task.QuartzTaskInfo;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskState;
import org.globalart.astrum.testdb.DatabaseTest;

import org.springframework.test.context.TestPropertySource;
import org.springframework.test.context.bean.override.mockito.MockitoBean;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Integration tests for {@link QuartzJobKeyUnificationTask} timeout behavior.
 */
@TestPropertySource(properties = {
    "astrum.quartz.jobkey.unification.timeout=2s",
    "astrum.quartz.jobkey.unification.check.interval=500ms"
})
class QuartzJobKeyUnificationTaskTimeoutTest
    extends QuartzJobKeyUnificationTaskTestSupport
{
  @MockitoBean
  private NodeHeartbeatManager heartbeatManager;

  @DatabaseTest
  void shouldTimeoutWhenRunningTaskBlocksMigration() throws Exception {
    try {
      // Create a legacy task
      QuartzTaskInfo legacyTask = createLegacyTask("Blocking Legacy Task");
      assertTaskLegacy(legacyTask);

      // Prepare the SleeperTask to block indefinitely when run
      SleeperTask.reset();

      // Run the legacy task - it will wait for meWait signal (which we won't send until after timeout)
      legacyTask.runNow();
      // Wait for task to signal "started"
      SleeperTask.youWait.await(3, TimeUnit.SECONDS);

      // Run migration task - it should timeout because the running task blocks migration
      TaskInfo migrationTask = runMigrationTask();

      // Wait for migration task to finish
      assertTaskState(WAIT_TIMEOUT, migrationTask, TaskState.OK);

      assertThat(migrationTask.getLastRunState().getEndState(), is(TaskState.FAILED));
      // Verify the error message for the last run contains expected text

      IllegalStateException expected =
          assertThrows(IllegalStateException.class, () -> ((QuartzTaskInfo) migrationTask).getTaskFuture().get());
      String errorMessage = expected.getMessage();

      assertThat(errorMessage, containsString("Timeout reached after 2 sec"));
      assertThat(errorMessage, containsString("1 tasks still need migration (running or blocked)"));
      assertThat(errorMessage, containsString("Task will retry on next node startup"));
      // Fix 1 verification: old nodes were not the blocker — message must not blame them
      assertThat(errorMessage, not(containsString("Old nodes still running in cluster")));
    }
    finally {
      // Let the blocking task complete to clean up
      SleeperTask.meWait.countDown();
    }
  }

  @DatabaseTest
  void shouldTimeoutWhenOldNodesBlockMigration() throws Exception {
    // Simulate two nodes running different versions — makes oldNodesRunning() return true
    configureOldNodesRunning();

    // No legacy tasks: notMigrated == 0, but oldNodesRunning == true → loop never breaks
    TaskInfo migrationTask = runMigrationTask();

    assertTaskState(WAIT_TIMEOUT, migrationTask, TaskState.OK);
    assertThat(migrationTask.getLastRunState().getEndState(), is(TaskState.FAILED));

    IllegalStateException expected =
        assertThrows(IllegalStateException.class, () -> ((QuartzTaskInfo) migrationTask).getTaskFuture().get());
    String errorMessage = expected.getMessage();

    assertThat(errorMessage, containsString("Timeout reached after 2 sec"));
    assertThat(errorMessage, containsString("Old nodes still running in cluster"));
    assertThat(errorMessage, containsString("Task will retry on next node startup"));
    // notMigrated was 0 — message must not mention task migration count
    assertThat(errorMessage, not(containsString("tasks still need migration")));
  }

  @DatabaseTest
  void shouldTimeoutWithBothConditions() throws Exception {
    // Simulate old nodes running AND a running legacy task so both conditions are true
    configureOldNodesRunning();

    try {
      QuartzTaskInfo legacyTask = createLegacyTask("Blocking Legacy Task");
      assertTaskLegacy(legacyTask);

      SleeperTask.reset();
      legacyTask.runNow();
      SleeperTask.youWait.await(3, TimeUnit.SECONDS);

      TaskInfo migrationTask = runMigrationTask();

      assertTaskState(WAIT_TIMEOUT, migrationTask, TaskState.OK);
      assertThat(migrationTask.getLastRunState().getEndState(), is(TaskState.FAILED));

      IllegalStateException expected =
          assertThrows(IllegalStateException.class, () -> ((QuartzTaskInfo) migrationTask).getTaskFuture().get());
      String errorMessage = expected.getMessage();

      assertThat(errorMessage, containsString("Timeout reached after 2 sec"));
      assertThat(errorMessage, containsString("tasks still need migration (running or blocked)"));
      assertThat(errorMessage, containsString("Old nodes still running in cluster"));
      assertThat(errorMessage, containsString("Task will retry on next node startup"));
    }
    finally {
      SleeperTask.meWait.countDown();
    }
  }

  private void configureOldNodesRunning() {
    Map<String, Object> systemInfo1 = new HashMap<>();
    systemInfo1.put("astrum-status", Map.of("version", "3.93.0-SNAPSHOT", "buildRevision", "abc123"));
    NodeHeartbeat currentNode = mock(NodeHeartbeat.class);
    when(currentNode.systemInfo()).thenReturn(systemInfo1);

    Map<String, Object> systemInfo2 = new HashMap<>();
    systemInfo2.put("astrum-status", Map.of("version", "3.88.0", "buildRevision", "old456"));
    NodeHeartbeat oldNode = mock(NodeHeartbeat.class);
    when(oldNode.systemInfo()).thenReturn(systemInfo2);

    when(heartbeatManager.getActiveNodeHeartbeatData()).thenReturn(List.of(currentNode, oldNode));
  }
}
