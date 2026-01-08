
package org.globalart.astrum.scheduling;

import java.util.List;

import org.globalart.astrum.formfields.FormField;

/**
 * Descriptor for a {@link Task}.
 *
 * Descriptors should be named singleton components.
 *
 * @since 3.0
 */
public interface TaskDescriptor
{
  /**
   * Unique identifier for descriptor.
   */
  String getId();

  /**
   * Short descriptive name of task.
   */
  String getName();

  /**
   * The type of task.
   */
  Class<? extends Task> getType();

  /**
   * Task configuration fields.
   */
  List<FormField> getFormFields();

  /**
   * @since 3.27
   */
  TaskConfiguration createTaskConfiguration();

  /**
   * Directly manipulate task configuration before storing
   *
   * @since 3.2
   * @param configuration task's configuration
   */
  void initializeConfiguration(TaskConfiguration configuration);

  // TODO: Figure out some clearer terms to use for the following state flags:

  /**
   * Returns task visibility.
   *
   * Visible tasks are shown to users.
   *
   * May be overridden by {@link TaskConfiguration#setVisible(boolean)}.
   */
  boolean isVisible();

  /**
   * Returns task exposure.
   *
   * Exposed tasks are allowed to be created by users.
   */
  boolean isExposed();

  /**
   * Returns true if the job should be marked as recoverable
   *
   * @since 3.15
   */
  boolean isRecoverable();

  boolean allowConcurrentRun();

  boolean isReadOnlyUi();

  /**
   * Allows the descriptor to complete the configuration of the task before it is stored.
   *
   * @param config Instance of the task configuration to be completed.
   */
  void completeConfiguration(TaskConfiguration config);

  /**
   * Returns {@code true} if at most one instance of this task type should exist in the scheduler at any time.
   * When {@code true}, the task scheduler will reject a second scheduling request if a task of this type
   * already exists in the job store.
   */
  default boolean isSingletonTaskType() {
    return false;
  }
}
