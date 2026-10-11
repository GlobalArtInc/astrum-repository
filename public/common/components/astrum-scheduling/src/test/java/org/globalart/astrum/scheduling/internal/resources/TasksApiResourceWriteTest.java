package org.globalart.astrum.scheduling.internal.resources;

import java.util.Date;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import jakarta.validation.ValidationException;
import jakarta.ws.rs.NotFoundException;
import jakarta.ws.rs.WebApplicationException;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.formfields.NumberTextFormField;
import org.globalart.astrum.formfields.StringTextFormField;
import org.globalart.astrum.rest.ValidationErrorXO;
import org.globalart.astrum.rest.ValidationErrorsException;
import org.globalart.astrum.rest.WebApplicationMessageException;
import org.globalart.astrum.scheduling.CurrentState;
import org.globalart.astrum.scheduling.ExternalTaskState;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskDescriptor;
import org.globalart.astrum.scheduling.TaskFactory;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskNotificationCondition;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.TaskState;
import org.globalart.astrum.scheduling.TaskUtils;
import org.globalart.astrum.scheduling.api.FrequencyXO;
import org.globalart.astrum.scheduling.api.TaskFormFieldXO;
import org.globalart.astrum.scheduling.api.TaskTemplateXO;
import org.globalart.astrum.scheduling.api.TaskXO;
import org.globalart.astrum.scheduling.schedule.Cron;
import org.globalart.astrum.scheduling.schedule.Daily;
import org.globalart.astrum.scheduling.schedule.Manual;
import org.globalart.astrum.scheduling.schedule.Schedule;
import org.globalart.astrum.scheduling.schedule.ScheduleFactoryImpl;

import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;
import org.globalart.astrum.testcommon.validation.ValidationExtension;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyMap;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith({ValidationExtension.class, AuthenticationExtension.class, MockitoExtension.class})
@WithUser
class TasksApiResourceWriteTest
{
  private static final String TYPE = "blobstore.compact";

  @Mock
  private TaskScheduler taskScheduler;

  @Mock
  private TaskFactory taskFactory;

  @Mock
  private TaskUtils taskUtils;

  @Mock
  private TaskDescriptor descriptor;

  private TasksApiResource underTest;

  @BeforeEach
  void setup() {
    underTest = new TasksApiResource(taskScheduler, taskUtils, false);
    lenient().when(taskScheduler.getTaskFactory()).thenReturn(taskFactory);
    lenient().when(taskScheduler.getScheduleFactory()).thenReturn(new ScheduleFactoryImpl()
    {
      @Override
      public Cron cron(final Date startAt, final String cronExpression) {
        if (!cronExpression.matches("[0-9*?/ ]+")) {
          throw new IllegalArgumentException("Invalid Cron expression: " + cronExpression);
        }
        return super.cron(startAt, cronExpression);
      }
    });
    lenient().when(taskScheduler.createTaskConfigurationInstance(anyString())).thenAnswer(invocation -> {
      TaskConfiguration configuration = new TaskConfiguration();
      configuration.setTypeId(invocation.getArgument(0));
      return configuration;
    });
    lenient().when(taskScheduler.toExternalTaskState(any()))
        .thenReturn(new ExternalTaskState(TaskState.WAITING, null, null, null, null, null));
    lenient().when(descriptor.getId()).thenReturn(TYPE);
    lenient().when(descriptor.getName()).thenReturn("Admin - Compact blob store");
    lenient().when(descriptor.isExposed()).thenReturn(true);
    lenient().when(descriptor.allowConcurrentRun()).thenReturn(false);
    List<FormField> fields = List.of(
        new StringTextFormField("blobstoreName", "Blob store", "The blob store", FormField.MANDATORY),
        new NumberTextFormField("days", "Days", "Days to keep", FormField.OPTIONAL)
            .withInitialValue(7).withMinimumValue(1).withMaximumValue(30));
    lenient().when(descriptor.getFormFields()).thenReturn(fields);
    lenient().when(taskFactory.findDescriptor(TYPE)).thenReturn(descriptor);
  }

  @Test
  void templatesListOnlyTheTypesThatCanBeCreated() {
    TaskDescriptor hidden = mock(TaskDescriptor.class);
    when(hidden.isExposed()).thenReturn(false);
    when(taskFactory.getDescriptors()).thenReturn(List.of(descriptor, hidden));

    List<TaskTemplateXO> templates = underTest.getTemplates();

    assertThat(templates.stream().map(TaskTemplateXO::getType).toList(), contains(TYPE));
  }

  @Test
  void templateCarriesTheFormFieldsAndTheirInitialValues() {
    TaskTemplateXO template = underTest.getTemplate(TYPE);

    assertThat(template.getName(), is("Admin - Compact blob store"));
    assertThat(template.getEnabled(), is(true));
    assertThat(template.getConcurrentRun(), is(false));
    assertThat(template.getFrequency().getSchedule(), is("manual"));
    assertThat(template.getNotificationCondition(), is(TaskNotificationCondition.FAILURE));
    assertThat(template.getProperties(), hasEntry("blobstoreName", ""));
    assertThat(template.getProperties(), hasEntry("days", "7"));
    assertThat(template.getFormFields().stream().map(TaskFormFieldXO::getId).toList(),
        contains("blobstoreName", "days"));
    TaskFormFieldXO days = template.getFormFields().get(1);
    assertThat(days.getType(), is("number"));
    assertThat(days.isRequired(), is(false));
    assertThat(days.getMinValue(), is("1"));
    assertThat(days.getMaxValue(), is("30"));
    assertThat(template.getFormFields().get(0).isRequired(), is(true));
  }

  @Test
  void templateOfAnUnknownTypeIsNotFound() {
    assertThrows(NotFoundException.class, () -> underTest.getTemplate("nosuchtype"));
  }

  @Test
  void createSchedulesTheTask() {
    TaskInfo created = taskInfo("task-1", TYPE, new TaskConfiguration());
    when(taskScheduler.scheduleTask(any(), any())).thenReturn(created);

    TaskTemplateXO template = template(TYPE, "Compact default");
    template.getProperties().put("blobstoreName", "default");
    template.getProperties().put(".id", "hijacked");
    template.setAlertEmail(" admin@example.com ");
    template.setNotificationCondition(TaskNotificationCondition.SUCCESS_FAILURE);
    template.setFrequency(frequency("daily", System.currentTimeMillis() + 3_600_000L));

    Response response = underTest.createTask(template);

    assertThat(response.getStatus(), is(201));
    assertThat(((TaskXO) response.getEntity()).getId(), is("task-1"));
    ArgumentCaptor<TaskConfiguration> configuration = ArgumentCaptor.forClass(TaskConfiguration.class);
    ArgumentCaptor<Schedule> schedule = ArgumentCaptor.forClass(Schedule.class);
    verify(taskScheduler).scheduleTask(configuration.capture(), schedule.capture());
    assertThat(configuration.getValue().getName(), is("Compact default"));
    assertThat(configuration.getValue().getTypeId(), is(TYPE));
    assertThat(configuration.getValue().isEnabled(), is(true));
    assertThat(configuration.getValue().getString("blobstoreName"), is("default"));
    assertThat(configuration.getValue().getId(), not("hijacked"));
    assertThat(configuration.getValue().getAlertEmail(), is("admin@example.com"));
    assertThat(configuration.getValue().getNotificationCondition(), is(TaskNotificationCondition.SUCCESS_FAILURE));
    assertThat(schedule.getValue(), instanceOf(Daily.class));
    verify(taskUtils).validateTaskCreationForAPI(eq("Compact default"), eq(TYPE), anyMap(), any(Daily.class));
  }

  @Test
  void createWithoutFrequencyRunsManually() {
    TaskInfo created = taskInfo("task-1", TYPE, new TaskConfiguration());
    when(taskScheduler.scheduleTask(any(), any())).thenReturn(created);

    underTest.createTask(template(TYPE, "Compact default"));

    verify(taskScheduler).scheduleTask(any(), any(Manual.class));
  }

  @Test
  void createRejectsMissingTypeNameAndUnknownTypes() {
    assertValidationError(() -> underTest.createTask(template(null, "A task")), "type");
    assertValidationError(() -> underTest.createTask(template("nosuchtype", "A task")), "type");
    assertValidationError(() -> underTest.createTask(template(TYPE, " ")), "name");

    TaskDescriptor hidden = mock(TaskDescriptor.class);
    when(hidden.isExposed()).thenReturn(false);
    when(taskFactory.findDescriptor("hidden")).thenReturn(hidden);
    assertValidationError(() -> underTest.createTask(template("hidden", "A task")), "type");

    verify(taskScheduler, never()).scheduleTask(any(), any());
  }

  @Test
  void createRejectsAnInvalidSchedule() {
    TaskTemplateXO template = template(TYPE, "Compact default");
    template.setFrequency(frequency("daily", null));
    assertValidationError(() -> underTest.createTask(template), "frequency.startDate");

    template.setFrequency(frequency("once", System.currentTimeMillis() - 60_000L));
    assertValidationError(() -> underTest.createTask(template), "frequency.startDate");

    FrequencyXO cron = frequency("cron", null);
    cron.setCronExpression("not a cron expression");
    template.setFrequency(cron);
    assertValidationError(() -> underTest.createTask(template), "frequency.cronExpression");

    verify(taskScheduler, never()).scheduleTask(any(), any());
  }

  @Test
  void createAnswersAConfigurationTheTypeRejectsWithAValidationError() {
    when(taskScheduler.scheduleTask(any(), any())).thenThrow(new IllegalArgumentException("Blob store is required"));

    ValidationErrorsException error =
        assertThrows(ValidationErrorsException.class, () -> underTest.createTask(template(TYPE, "Compact default")));

    assertThat(error.getValidationErrors().get(0).getMessage(), is("Blob store is required"));
  }

  @Test
  void createPassesOnADuplicateTask() {
    doThrow(new WebApplicationMessageException(Response.Status.CONFLICT, "Task with name 'x' already exists",
        "application/json"))
        .when(taskUtils).validateTaskCreationForAPI(anyString(), anyString(), anyMap(), any());

    WebApplicationException error =
        assertThrows(WebApplicationException.class, () -> underTest.createTask(template(TYPE, "x")));

    assertThat(error.getResponse().getStatus(), is(409));
    verify(taskScheduler, never()).scheduleTask(any(), any());
  }

  @Test
  void updateMergesTheChangesIntoTheStoredConfiguration() {
    TaskConfiguration stored = new TaskConfiguration();
    stored.setId("task-1");
    stored.setTypeId(TYPE);
    stored.setName("Compact default");
    stored.setEnabled(true);
    stored.setAlertEmail("old@example.com");
    stored.setString("blobstoreName", "default");
    stored.setString("days", "7");
    TaskInfo task = taskInfo("task-1", TYPE, stored);
    when(task.getSchedule()).thenReturn(new ScheduleFactoryImpl().manual());
    when(taskScheduler.getTaskById("task-1")).thenReturn(task);
    when(taskScheduler.scheduleTask(any(), any())).thenReturn(task);

    TaskTemplateXO update = new TaskTemplateXO();
    update.setEnabled(false);
    update.setProperties(new HashMap<>(Map.of("days", "14")));
    FrequencyXO cron = frequency("cron", null);
    cron.setCronExpression("0 0 1 * * ?");
    cron.setTimeZoneOffset("+03:00");
    update.setFrequency(cron);

    underTest.updateTask("task-1", update);

    ArgumentCaptor<TaskConfiguration> configuration = ArgumentCaptor.forClass(TaskConfiguration.class);
    ArgumentCaptor<Schedule> schedule = ArgumentCaptor.forClass(Schedule.class);
    verify(taskScheduler).scheduleTask(configuration.capture(), schedule.capture());
    assertThat(configuration.getValue().getId(), is("task-1"));
    assertThat(configuration.getValue().getName(), is("Compact default"));
    assertThat(configuration.getValue().isEnabled(), is(false));
    assertThat(configuration.getValue().getString("blobstoreName"), is("default"));
    assertThat(configuration.getValue().getString("days"), is("14"));
    assertThat(configuration.getValue().getAlertEmail(), is("old@example.com"));
    assertThat(schedule.getValue(), instanceOf(Cron.class));
    assertThat(((Cron) schedule.getValue()).getCronExpression(), is("0 0 1 * * ?"));
  }

  @Test
  void updateClearsTheAlertEmailWhenItIsSentEmpty() {
    TaskConfiguration stored = new TaskConfiguration();
    stored.setId("task-1");
    stored.setTypeId(TYPE);
    stored.setName("Compact default");
    stored.setAlertEmail("old@example.com");
    TaskInfo task = taskInfo("task-1", TYPE, stored);
    when(taskScheduler.getTaskById("task-1")).thenReturn(task);
    when(taskScheduler.scheduleTask(any(), any())).thenReturn(task);

    TaskTemplateXO update = new TaskTemplateXO();
    update.setAlertEmail("");
    underTest.updateTask("task-1", update);

    ArgumentCaptor<TaskConfiguration> configuration = ArgumentCaptor.forClass(TaskConfiguration.class);
    verify(taskScheduler).scheduleTask(configuration.capture(), any());
    assertThat(configuration.getValue().getAlertEmail(), nullValue());
  }

  @Test
  void updateWithoutFrequencyKeepsTheSchedule() {
    TaskConfiguration stored = new TaskConfiguration();
    stored.setId("task-1");
    stored.setTypeId(TYPE);
    stored.setName("Compact default");
    TaskInfo task = taskInfo("task-1", TYPE, stored);
    Schedule daily = new ScheduleFactoryImpl().daily(new Date());
    when(task.getSchedule()).thenReturn(daily);
    when(taskScheduler.getTaskById("task-1")).thenReturn(task);
    when(taskScheduler.scheduleTask(any(), any())).thenReturn(task);

    underTest.updateTask("task-1", new TaskTemplateXO());

    verify(taskScheduler).scheduleTask(any(), eq(daily));
  }

  @Test
  void updateRejectsAChangedTypeARunningTaskAndUnknownIds() {
    TaskConfiguration stored = new TaskConfiguration();
    stored.setTypeId(TYPE);
    TaskInfo task = taskInfo("task-1", TYPE, stored);
    when(taskScheduler.getTaskById("task-1")).thenReturn(task);

    TaskTemplateXO otherType = new TaskTemplateXO();
    otherType.setType("other");
    assertValidationError(() -> underTest.updateTask("task-1", otherType), "type");

    when(taskScheduler.toExternalTaskState(task))
        .thenReturn(new ExternalTaskState(TaskState.RUNNING, null, null, null, null, null));
    WebApplicationException running =
        assertThrows(WebApplicationException.class, () -> underTest.updateTask("task-1", new TaskTemplateXO()));
    assertThat(running.getResponse().getStatus(), is(409));

    assertThrows(NotFoundException.class, () -> underTest.updateTask("nosuchtask", new TaskTemplateXO()));
    verify(taskScheduler, never()).scheduleTask(any(), any());
  }

  @Test
  void updateDoesNotChangeTheSourceOfAScriptUnlessScriptsMayBeCreated() {
    TaskConfiguration stored = new TaskConfiguration();
    stored.setTypeId("script");
    stored.setString("source", "log.info('hello')");
    TaskInfo task = taskInfo("task-1", "script", stored);
    when(taskScheduler.getTaskById("task-1")).thenReturn(task);

    TaskTemplateXO update = new TaskTemplateXO();
    update.setProperties(Map.of("source", "log.info('changed')"));

    assertValidationError(() -> underTest.updateTask("task-1", update), "properties.source");
  }

  @Test
  void updateAnswersAConfigurationTheTypeRejectsWithAValidationError() {
    TaskConfiguration stored = new TaskConfiguration();
    stored.setId("task-1");
    stored.setTypeId(TYPE);
    stored.setName("Compact default");
    TaskInfo task = taskInfo("task-1", TYPE, stored);
    when(taskScheduler.getTaskById("task-1")).thenReturn(task);
    when(taskScheduler.scheduleTask(any(), any())).thenThrow(new ValidationException("Invalid blob store"));

    assertThrows(ValidationErrorsException.class, () -> underTest.updateTask("task-1", new TaskTemplateXO()));
  }

  @Test
  void deleteRemovesTheTask() {
    TaskInfo task = taskInfo("task-1", TYPE, new TaskConfiguration());
    when(taskScheduler.getTaskById("task-1")).thenReturn(task);

    underTest.deleteTask("task-1");

    verify(task).remove();
  }

  @Test
  void deleteOfAnUnknownOrInternalTaskIsNotFound() {
    TaskConfiguration internal = new TaskConfiguration();
    internal.setVisible(false);
    TaskInfo task = taskInfo("task-1", TYPE, internal);
    when(taskScheduler.getTaskById("task-1")).thenReturn(task);

    assertThrows(NotFoundException.class, () -> underTest.deleteTask("task-1"));
    assertThrows(NotFoundException.class, () -> underTest.deleteTask("nosuchtask"));
    verify(task, never()).remove();
  }

  private static void assertValidationError(final Runnable call, final String id) {
    ValidationErrorsException error = assertThrows(ValidationErrorsException.class, call::run);
    assertThat(error.getValidationErrors().stream().map(ValidationErrorXO::getId).toList(), contains(id));
  }

  private static TaskTemplateXO template(final String type, final String name) {
    TaskTemplateXO template = new TaskTemplateXO();
    template.setType(type);
    template.setName(name);
    template.setEnabled(true);
    template.setProperties(new HashMap<>());
    return template;
  }

  private static FrequencyXO frequency(final String schedule, final Long startDate) {
    FrequencyXO frequency = new FrequencyXO();
    frequency.setSchedule(schedule);
    frequency.setStartDate(startDate);
    frequency.setTimeZoneOffset("+00:00");
    return frequency;
  }

  private static TaskInfo taskInfo(final String id, final String typeId, final TaskConfiguration configuration) {
    TaskInfo task = mock(TaskInfo.class);
    lenient().when(task.getId()).thenReturn(id);
    lenient().when(task.getTypeId()).thenReturn(typeId);
    lenient().when(task.getName()).thenReturn(configuration.getName());
    lenient().when(task.getConfiguration()).thenReturn(configuration);
    lenient().when(task.getCurrentState()).thenReturn(mock(CurrentState.class));
    return task;
  }
}
