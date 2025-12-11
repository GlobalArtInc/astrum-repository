
package org.globalart.astrum.internal.security.secrets.task;

import java.util.List;
import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.logging.task.TaskLogType;
import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.TaskSupport;
import org.globalart.astrum.security.secrets.SecretsMigrator;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Task to migrate secrets from external sources to a single source of truth
 */
@Component
@TaskLogging(TaskLogType.TASK_LOG_ONLY)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class SecretsMigrationTask
    extends TaskSupport
    implements Cancelable
{
  private final List<SecretsMigrator> migrators;

  @Autowired
  public SecretsMigrationTask(final List<SecretsMigrator> migrators) {
    this.migrators = checkNotNull(migrators);
  }

  @Override
  public String getMessage() {
    return "Migrate existing secrets into a single source (secrets table).";
  }

  @Override
  protected Object execute() throws Exception {
    for (SecretsMigrator migrator : migrators) {
      migrator.migrate();
    }
    return null;
  }
}
