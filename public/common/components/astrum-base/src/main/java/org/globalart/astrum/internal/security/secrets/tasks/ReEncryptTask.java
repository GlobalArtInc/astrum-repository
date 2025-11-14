
package org.globalart.astrum.internal.security.secrets.tasks;

import java.time.Duration;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;

import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.crypto.secrets.SecretData;
import org.globalart.astrum.crypto.secrets.SecretsService;
import org.globalart.astrum.crypto.secrets.SecretsStore;
import org.globalart.astrum.logging.task.ProgressLogIntervalHelper;
import org.globalart.astrum.logging.task.TaskLogType;
import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.TaskSupport;

import org.springframework.beans.factory.annotation.Value;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.scheduling.CancelableHelper.checkCancellation;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

@Component
@Qualifier(ReEncryptTaskDescriptor.TYPE_ID)
@TaskLogging(TaskLogType.TASK_LOG_ONLY)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ReEncryptTask
    extends TaskSupport
    implements Cancelable
{
  private final SecretsService secretsService;

  private final SecretsStore secretsStore;

  private final long delayTimeMs;

  @Autowired
  public ReEncryptTask(
      final SecretsService secretsService,
      final SecretsStore secretsStore,
      @Value("${astrum.distributed.events.fetch.interval.seconds:5}") final int pollInterval)
  {
    this.secretsService = checkNotNull(secretsService);
    this.secretsStore = checkNotNull(secretsStore);
    this.delayTimeMs = Duration.ofSeconds(pollInterval).toMillis() * 2;
  }

  @Override
  public String getMessage() {
    return "Re-encrypting secrets with specified key id";
  }

  @Override
  protected Object execute() throws Exception {
    waitActiveKeyProcessing();
    log.info("Started re-encrypting secrets with provided keyId");
    String secretKeyId = taskConfiguration().getString("keyId");
    int processed = reEncrypt(secretKeyId);
    log.info("Completed re-encryption of secrets with keyId '{}'. Processed {} secrets", secretKeyId, processed);
    return processed;
  }

  private int reEncrypt(String keyId) {
    int processedCount = 0;
    List<SecretData> page = secretsStore.fetchWithDifferentKeyId(keyId, Continuations.BROWSE_LIMIT);
    try (ProgressLogIntervalHelper progress = new ProgressLogIntervalHelper(log, 60)) {
      while (!page.isEmpty()) {
        page.forEach(secret -> {
          checkCancellation();
          secretsService.reEncrypt(secret, keyId);
        });
        processedCount += page.size();
        progress.info("Processed {} secrets.", processedCount);
        page = secretsStore.fetchWithDifferentKeyId(keyId, Continuations.BROWSE_LIMIT);
      }
    }
    return processedCount;
  }

  /**
   * Delay to wait for other nodes to process the event that sets the new active key
   */
  private void waitActiveKeyProcessing() {
    try {
      Thread.sleep(delayTimeMs);
    }
    catch (InterruptedException e) {
      // ignore
    }
  }

}
