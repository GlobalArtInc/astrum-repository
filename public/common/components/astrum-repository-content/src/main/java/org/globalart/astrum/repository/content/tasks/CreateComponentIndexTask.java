
package org.globalart.astrum.repository.content.tasks;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.logging.task.TaskLogType;
import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.TaskInterruptedException;
import org.globalart.astrum.scheduling.TaskSupport;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.DISABLE_CREATING_COMPONENT_INDEXES_TASK;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * System task to populate the {format}_component tables
 */
@Component
@TaskLogging(TaskLogType.TASK_LOG_ONLY_WITH_PROGRESS)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CreateComponentIndexTask
    extends TaskSupport
    implements Cancelable
{
  private static final Logger LOG = LoggerFactory.getLogger(CreateComponentIndexTask.class);

  private final CreateComponentIndexService createComponentIndexService;

  private final boolean disableTask;

  @Autowired
  public CreateComponentIndexTask(
      final CreateComponentIndexService createComponentIndexService,
      @Value("${" + DISABLE_CREATING_COMPONENT_INDEXES_TASK + ":false}") final boolean disableTask)
  {
    this.createComponentIndexService = checkNotNull(createComponentIndexService);
    this.disableTask = disableTask;
  }

  @Override
  public String getMessage() {
    return "Create new indexes for querying the various format component tables by full coordinates";
  }

  @Override
  protected Object execute() throws Exception {
    if (disableTask) {
      throw new TaskInterruptedException("The component table index task was disabled", disableTask);
    }

    createComponentIndexService.recreateComponentIndexes();

    return null;
  }
}
