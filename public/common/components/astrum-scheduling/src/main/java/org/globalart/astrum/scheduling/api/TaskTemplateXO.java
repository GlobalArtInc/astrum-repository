package org.globalart.astrum.scheduling.api;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.formfields.FormField;
import org.globalart.astrum.scheduling.TaskDescriptor;
import org.globalart.astrum.scheduling.TaskNotificationCondition;

import io.swagger.v3.oas.annotations.media.Schema;

import static java.util.stream.Collectors.toList;

/**
 * A task as a client sends it to create or update one, and the template of a task type it starts from.
 *
 * @since 3.96
 */
@Schema(description = "The settings of a task to create or update, or the template of a task type")
public class TaskTemplateXO
{
  @Schema(description = "The type identifier of the task; it cannot be changed by an update",
      example = "blobstore.compact")
  private String type;

  @Schema(description = "The name of the task", example = "Compact default blob store")
  private String name;

  @Schema(description = "Whether the task is enabled", example = "true")
  private Boolean enabled;

  @Schema(description = "Email address to send notifications to", example = "admin@example.com")
  private String alertEmail;

  @Schema(description = "Condition for sending notifications", example = "FAILURE")
  private TaskNotificationCondition notificationCondition;

  private FrequencyXO frequency;

  @Schema(description = "Task-specific configuration properties")
  private Map<String, String> properties;

  @Schema(description = "The settings of the task type; only sent in templates",
      accessMode = Schema.AccessMode.READ_ONLY)
  private List<TaskFormFieldXO> formFields;

  @Schema(description = "Whether a task of this type may run while another one runs; only sent in templates",
      accessMode = Schema.AccessMode.READ_ONLY)
  private Boolean concurrentRun;

  /**
   * The template a new task of the given type starts from: enabled, run manually, with the initial values of the
   * type's settings.
   */
  public static TaskTemplateXO fromDescriptor(final TaskDescriptor descriptor) {
    TaskTemplateXO template = new TaskTemplateXO();
    template.setType(descriptor.getId());
    template.setName(descriptor.getName());
    template.setEnabled(true);
    template.setNotificationCondition(TaskNotificationCondition.DEFAULT);
    FrequencyXO frequency = new FrequencyXO();
    frequency.setSchedule("manual");
    template.setFrequency(frequency);
    template.setConcurrentRun(descriptor.allowConcurrentRun());

    List<FormField> fields = descriptor.getFormFields() == null ? List.of() : descriptor.getFormFields();
    Map<String, String> properties = new LinkedHashMap<>();
    for (FormField<?> field : fields) {
      properties.put(field.getId(), field.getInitialValue() == null ? "" : field.getInitialValue().toString());
    }
    template.setProperties(properties);
    template.setFormFields(fields.stream().map(field -> TaskFormFieldXO.create(field)).collect(toList()));
    return template;
  }

  public String getType() {
    return type;
  }

  public void setType(final String type) {
    this.type = type;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public Boolean getEnabled() {
    return enabled;
  }

  public void setEnabled(final Boolean enabled) {
    this.enabled = enabled;
  }

  public String getAlertEmail() {
    return alertEmail;
  }

  public void setAlertEmail(final String alertEmail) {
    this.alertEmail = alertEmail;
  }

  public TaskNotificationCondition getNotificationCondition() {
    return notificationCondition;
  }

  public void setNotificationCondition(final TaskNotificationCondition notificationCondition) {
    this.notificationCondition = notificationCondition;
  }

  public FrequencyXO getFrequency() {
    return frequency;
  }

  public void setFrequency(final FrequencyXO frequency) {
    this.frequency = frequency;
  }

  public Map<String, String> getProperties() {
    return properties;
  }

  public void setProperties(final Map<String, String> properties) {
    this.properties = properties;
  }

  public List<TaskFormFieldXO> getFormFields() {
    return formFields;
  }

  public void setFormFields(final List<TaskFormFieldXO> formFields) {
    this.formFields = formFields;
  }

  public Boolean getConcurrentRun() {
    return concurrentRun;
  }

  public void setConcurrentRun(final Boolean concurrentRun) {
    this.concurrentRun = concurrentRun;
  }

  @Override
  public String toString() {
    return "TaskTemplateXO{" +
        "type='" + type + '\'' +
        ", name='" + name + '\'' +
        ", enabled=" + enabled +
        ", notificationCondition=" + notificationCondition +
        ", frequency=" + frequency +
        ", properties=" + (properties == null ? null : properties.keySet()) +
        '}';
  }
}
