
package org.globalart.astrum.internal.security.secrets;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.internal.security.secrets.tasks.ReEncryptTaskDescriptor;
import org.globalart.astrum.scheduling.TaskScheduler;

import com.codahale.metrics.health.HealthCheck;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Health check to determine if re-encryption is required.
 */
@Component
@Qualifier("Re-encryption required")
public class ReEncryptionRequiredHealthCheck
    extends HealthCheck
{
  private static final String HEALTHY_MESSAGE = "All secrets using same encryption key. Re-encryption is not required.";

  private static final String RE_ENCRYPTION_IN_PROGRESS_MESSAGE =
      "Re-encryption in progress. Check task logs for more information.";

  private static final String UNHEALTHY_MESSAGE = "Detected more than one encryption key in use. " +
      "Re-encryption is required. See help documentation for information on how to start re-encryption.";

  private final SecretsService secretsService;

  private final TaskScheduler taskScheduler;

  @Autowired
  public ReEncryptionRequiredHealthCheck(final SecretsService secretsService, final TaskScheduler taskScheduler) {
    this.secretsService = checkNotNull(secretsService);
    this.taskScheduler = checkNotNull(taskScheduler);
  }

  @Override
  protected Result check() throws Exception {
    if (isReEncryptTaskRunning()) {
      return Result.healthy(RE_ENCRYPTION_IN_PROGRESS_MESSAGE);
    }
    else if (secretsService.isReEncryptRequired()) {
      return Result.unhealthy(UNHEALTHY_MESSAGE);
    }
    return Result.healthy(HEALTHY_MESSAGE);
  }

  private boolean isReEncryptTaskRunning() {
    return taskScheduler.getTaskByTypeId(ReEncryptTaskDescriptor.TYPE_ID) != null;
  }
}
