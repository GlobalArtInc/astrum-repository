
package org.globalart.astrum.scheduling.internal.resources;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.Date;
import java.util.List;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.Future;
import java.util.function.Function;

import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response.Status;

import org.globalart.astrum.rest.Page;
import org.globalart.astrum.scheduling.CurrentState;
import org.globalart.astrum.scheduling.ExternalTaskState;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.TaskState;
import org.globalart.astrum.scheduling.api.TaskXO;
import org.globalart.astrum.scheduling.schedule.Schedule;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.invocation.InvocationOnMock;
import org.mockito.junit.jupiter.MockitoExtension;

import static jakarta.ws.rs.core.Response.Status.CONFLICT;
import static jakarta.ws.rs.core.Response.Status.INTERNAL_SERVER_ERROR;
import static jakarta.ws.rs.core.Response.Status.NOT_FOUND;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.scheduling.TaskState.OK;
import static org.globalart.astrum.scheduling.TaskState.RUNNING;
import static org.globalart.astrum.scheduling.TaskState.WAITING;

@ExtendWith({ValidationExtension.class, AuthenticationExtension.class, MockitoExtension.class})
@WithUser
class TasksApiResourceTest
{
  private final TaskConfiguration visibleConfig = configuration(true);

  private final TaskConfiguration invisibleConfig = configuration(false);

  private TestTaskInfo[] testTasks = new TestTaskInfo[]{
      new TestTaskInfo("task0", "Invisible task", "aType", new TestCurrentState(WAITING), invisibleConfig),
      new TestTaskInfo("task1", "Task 1", "anotherType", new TestCurrentState(WAITING),
          configuration(true, "Another Task Type")),
      new TestTaskInfo("task2", "Task 2", "aType", new TestCurrentState(RUNNING, new Date(), new CompletableFuture<>()),
          configuration(true, "A Task Type")),
      new TestTaskInfo("task3", "Task 3", "anotherType",
          new TestCurrentState(OK, new Date(), CompletableFuture.completedFuture(null)),
          configuration(true, "Another Task Type"))
  };

  @Mock
  private TaskScheduler taskScheduler;

  @InjectMocks
  TasksApiResource tasksResource;

  @BeforeEach
  void setup() {
    lenient().when(taskScheduler.getTaskById(any())).thenAnswer(this::findTask);
  }

  @Test
  void testResourceURI() {
    assertThat(TasksApiResource.RESOURCE_URI, is("/v1/tasks"));
  }

  /*
   * getTasks gets list of scheduled tasks
   */
  @Test
  void testGetTasks() {
    when(taskScheduler.listsTasks()).thenReturn(Arrays.asList(testTasks));
    when(taskScheduler.toExternalTaskState(any())).thenReturn(new ExternalTaskState(testTasks[1]),
        new ExternalTaskState(testTasks[2]), new ExternalTaskState(testTasks[3]));

    Page<TaskXO> page = tasksResource.getTasks(null);

    assertThat(page.getItems(), hasSize(3));
    assertThat(extract(page.getItems(), TaskXO::getId), contains("task1", "task2", "task3"));
    assertThat(extract(page.getItems(), TaskXO::getName), contains("Task 1", "Task 2", "Task 3"));
    assertThat(extract(page.getItems(), TaskXO::getType), contains("anotherType", "aType", "anotherType"));
    assertThat(extract(page.getItems(), TaskXO::getTypeName),
        contains("Another Task Type", "A Task Type", "Another Task Type"));
    assertThat(extract(page.getItems(), TaskXO::getCurrentState),
        contains(WAITING.toString(), RUNNING.toString(), OK.toString()));
  }

  /*
   * getTasks filters on task type
   */
  @Test
  void testGetTasks_taskTypeFilter() {
    when(taskScheduler.listsTasks()).thenReturn(Arrays.asList(testTasks));
    when(taskScheduler.toExternalTaskState(any())).thenReturn(new ExternalTaskState(testTasks[1]),
        new ExternalTaskState(testTasks[3]));
    Page<TaskXO> page = tasksResource.getTasks("anotherType");

    assertThat(page.getItems(), hasSize(2));
    assertThat(extract(page.getItems(), TaskXO::getId), contains("task1", "task3"));
  }

  /*
   * getTaskById gets tasks by id
   */
  @Test
  void testGetTaskById() {
    when(taskScheduler.toExternalTaskState(any())).thenReturn(new ExternalTaskState(testTasks[1]));

    TaskXO validTaskXO = tasksResource.getTaskById("task1");

    assertThat(validTaskXO.getId(), is("task1"));
    assertThat(validTaskXO.getName(), is("Task 1"));
    assertThat(validTaskXO.getType(), is("anotherType"));
    assertThat(validTaskXO.getTypeName(), is("Another Task Type"));
    assertThat(validTaskXO.getCurrentState(), is(WAITING.toString()));
  }

  /*
   * getTaskById returns properties, enabled, alertEmail, notificationCondition, and schedule details
   */
  @Test
  void testGetTaskById_includesNewFields() {
    TaskConfiguration configWithExtras = configuration(true);
    configWithExtras.setEnabled(false);
    configWithExtras.setAlertEmail("alert@example.com");
    configWithExtras.setNotificationCondition(org.globalart.astrum.scheduling.TaskNotificationCondition.SUCCESS_FAILURE);
    configWithExtras.setString("blobstoreName", "default");
    configWithExtras.setString("daysOlderThan", "3");

    TestTaskInfo taskWithExtras = new TestTaskInfo("task-extras", "Task with extras", "aType",
        new TestCurrentState(WAITING), configWithExtras);
    taskWithExtras.schedule = new org.globalart.astrum.scheduling.schedule.Once(new Date());

    when(taskScheduler.getTaskById("task-extras")).thenReturn(taskWithExtras);
    when(taskScheduler.toExternalTaskState(any())).thenReturn(new ExternalTaskState(taskWithExtras));

    TaskXO taskXO = tasksResource.getTaskById("task-extras");

    assertThat(taskXO.getProperties(), notNullValue());
    assertThat(taskXO.getProperties().get("blobstoreName"), is("default"));
    assertThat(taskXO.getProperties().get("daysOlderThan"), is("3"));
    assertThat(taskXO.isEnabled(), is(false));
    assertThat(taskXO.getAlertEmail(), is("alert@example.com"));
    assertThat(taskXO.getNotificationCondition(),
        is(org.globalart.astrum.scheduling.TaskNotificationCondition.SUCCESS_FAILURE));
    assertThat(taskXO.getSchedule(), is("once"));
    assertThat(taskXO.getStartDate(), notNullValue());
  }

  @Test
  void testGetTaskById_invalid() {
    WebApplicationException exception =
        assertThrows(WebApplicationException.class, () -> tasksResource.getTaskById("nosuchtask"));

    assertThat(Status.fromStatusCode(exception.getResponse().getStatus()), is(NOT_FOUND));
  }

  /*
   * run invokes runNow on tasks
   */
  @Test
  void testRun() {
    tasksResource.run("task1");

    assertThat(findTask("task1").runs, contains("REST API"));
  }

  @Test
  void testRun_invalidId() {
    WebApplicationException exception =
        assertThrows(WebApplicationException.class, () -> tasksResource.run("nosuchtask"));

    assertThat(Status.fromStatusCode(exception.getResponse().getStatus()), is(NOT_FOUND));
  }

  @Test
  void testRun_error() {
    when(taskScheduler.getTaskById(any())).thenThrow(new RuntimeException("error"));

    WebApplicationException exception = assertThrows(WebApplicationException.class, () -> tasksResource.run("error"));

    assertThat(Status.fromStatusCode(exception.getResponse().getStatus()), is(INTERNAL_SERVER_ERROR));
  }

  /*
   * stop cancels running tasks
   */
  @Test
  void testStop() {
    tasksResource.stop("task2");

    assertThat(findTask("task2").currentState.getFuture().isCancelled(), is(true));
  }

  /*
   * stop called with valid id for a non-running task
   */
  @Test
  void testStop_notRunning() {
    WebApplicationException exception = assertThrows(WebApplicationException.class, () -> tasksResource.stop("task1"));

    assertThat(Status.fromStatusCode(exception.getResponse().getStatus()), is(CONFLICT));

  }

  @Test
  void testStop_stopOnCompleted() {
    WebApplicationException exception = assertThrows(WebApplicationException.class, () -> tasksResource.stop("task3"));

    assertThat(Status.fromStatusCode(exception.getResponse().getStatus()), is(CONFLICT));
  }

  @Test
  void testStop_invalidId() {
    WebApplicationException exception =
        assertThrows(WebApplicationException.class, () -> tasksResource.stop("nosuchtask"));

    assertThat(Status.fromStatusCode(exception.getResponse().getStatus()), is(NOT_FOUND));
  }

  @Test
  void testStop_error() {
    when(taskScheduler.getTaskById(any())).thenThrow(new RuntimeException("error"));
    WebApplicationException exception = assertThrows(WebApplicationException.class, () -> tasksResource.stop("error"));

    assertThat(Status.fromStatusCode(exception.getResponse().getStatus()), is(INTERNAL_SERVER_ERROR));
  }

  private TestTaskInfo findTask(final InvocationOnMock invocation) {
    return findTask(invocation.getArguments()[0]);
  }

  private TestTaskInfo findTask(final Object id) {
    return Arrays.asList(testTasks)
        .stream()
        .filter(candidate -> candidate.getId().equals(id))
        .findFirst()
        .orElse(null);
  }

  private static TaskConfiguration configuration(final boolean visible) {
    TaskConfiguration configuration = new TaskConfiguration();
    configuration.setVisible(visible);
    return configuration;
  }

  private static TaskConfiguration configuration(final boolean visible, final String typeName) {
    TaskConfiguration configuration = new TaskConfiguration();
    configuration.setVisible(visible);
    configuration.setTypeName(typeName);
    return configuration;
  }

  private static <A, R> Collection<R> extract(final Collection<A> collection, final Function<A, R> fn) {
    return collection.stream().map(fn).toList();
  }

  private static class TestTaskInfo
      implements TaskInfo
  {
    String id;

    String name;

    String typeId;

    String message;

    String triggerSource;

    Object lastResult;

    CurrentState currentState;

    TaskConfiguration configuration;

    Schedule schedule;

    List<String> runs = new ArrayList<>();

    public TestTaskInfo(
        final String id,
        final String name,
        final String typeId,
        final CurrentState currentState,
        final TaskConfiguration configuration)
    {
      this.id = id;
      this.name = name;
      this.typeId = typeId;
      this.currentState = currentState;
      this.configuration = configuration;
    }

    @Override
    public boolean remove() {
      return false;
    }

    @Override
    public TaskInfo runNow(final String s) {
      runs.add(s);
      return this;
    }

    @Override
    public String getId() {
      return id;
    }

    @Override
    public String getName() {
      return name;
    }

    @Override
    public String getTypeId() {
      return typeId;
    }

    @Override
    public String getMessage() {
      return message;
    }

    @Override
    public String getTriggerSource() {
      return triggerSource;
    }

    @Override
    public Object getLastResult() {
      return lastResult;
    }

    @Override
    public CurrentState getCurrentState() {
      return currentState;
    }

    @Override
    public TaskConfiguration getConfiguration() {
      return configuration;
    }

    @Override
    public Schedule getSchedule() {
      return schedule;
    }

    public List<String> getRuns() {
      return runs;
    }
  }

  private static class TestCurrentState
      implements CurrentState
  {
    TaskState state;

    Date nextRun;

    Date runStarted;

    TaskState runState;

    Future<?> future;

    public TestCurrentState(final TaskState state, final Date runStarted, final Future<?> future) {
      this.state = state;
      this.runStarted = runStarted;
      this.future = future;
    }

    public TestCurrentState(final TaskState state) {
      this.state = state;
    }

    @Override
    public TaskState getState() {
      return state;
    }

    @Override
    public Date getNextRun() {
      return nextRun;
    }

    @Override
    public Date getRunStarted() {
      return runStarted;
    }

    @Override
    public TaskState getRunState() {
      return runState;
    }

    @Override
    public Future<?> getFuture() {
      return future;
    }
  }
}
