
package org.globalart.astrum.internal.security.secrets;

import java.util.Optional;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.cooperation2.Cooperation2;
import org.globalart.astrum.common.cooperation2.Cooperation2Factory;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.crypto.secrets.ActiveKeyChangeEvent;
import org.globalart.astrum.crypto.secrets.EncryptionKeyValidator;
import org.globalart.astrum.crypto.secrets.KeyAccessValidator;
import org.globalart.astrum.crypto.secrets.MissingKeyException;
import org.globalart.astrum.crypto.secrets.ReEncryptService;
import org.globalart.astrum.crypto.secrets.ReEncryptionNotSupportedException;
import org.globalart.astrum.internal.security.secrets.tasks.ReEncryptTaskDescriptor;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.security.UserIdHelper;

import org.springframework.beans.factory.annotation.Value;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.lang.String.format;
import static org.globalart.astrum.crypto.secrets.SecretsService.SECRETS_MIGRATION_VERSION;
import org.springframework.stereotype.Component;

/**
 * Implementation of {@link ReEncryptService}. It schedules a re-encryption task if one is not already running.
 */
@Component
public class ReEncryptServiceImpl
    implements ReEncryptService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String KEY_NOT_FOUND_ERR_MSG =
      "Key id '%s' not found. Check secrets configuration, make sure the file and key id exists.";

  private final KeyAccessValidator keyAccessValidator;

  private final EncryptionKeyValidator encryptionKeyValidator;

  private final TaskScheduler taskScheduler;

  private final EventManager eventManager;

  private final DatabaseCheck databaseCheck;

  private final Cooperation2 cooperation;

  @Autowired
  public ReEncryptServiceImpl(
      final KeyAccessValidator keyAccessValidator,
      final EncryptionKeyValidator encryptionKeyValidator,
      final TaskScheduler taskScheduler,
      final EventManager eventManager,
      final DatabaseCheck databaseCheck,
      final Cooperation2Factory cooperationFactory,
      @Value("${astrum.secrets.cooperation.enabled:true}") final boolean cooperationEnabled)
  {
    this.keyAccessValidator = checkNotNull(keyAccessValidator);
    this.encryptionKeyValidator = checkNotNull(encryptionKeyValidator);
    this.taskScheduler = checkNotNull(taskScheduler);
    this.eventManager = checkNotNull(eventManager);
    this.databaseCheck = checkNotNull(databaseCheck);
    this.cooperation = checkNotNull(cooperationFactory).configure()
        .enabled(cooperationEnabled)
        .build("re-encrypt-task");
  }

  private void checkKeyExists(final String secretKeyId) {
    if (!keyAccessValidator.isValidKey(secretKeyId)) {
      throw new MissingKeyException(format(KEY_NOT_FOUND_ERR_MSG, secretKeyId));
    }
  }

  @Override
  public String submitReEncryption(
      final String secretKeyId,
      @Nullable final String notifyEmail) throws MissingKeyException, ReEncryptionNotSupportedException
  {
    return submitTask(secretKeyId, notifyEmail);
  }

  private void checkReEncryptionSupported() {
    if (!databaseCheck.isAtLeast(SECRETS_MIGRATION_VERSION)) {
      throw new ReEncryptionNotSupportedException(
          format("Re-encryption is not supported. Please upgrade DB to version %s", SECRETS_MIGRATION_VERSION));
    }
  }

  private void checkTaskNotSubmitted() {
    if (taskScheduler.getTaskByTypeId(ReEncryptTaskDescriptor.TYPE_ID) != null) {
      throw new IllegalStateException("Re-encryption task is already running");
    }
  }

  /**
   * Schedules re-encrypt task if there is not an existing one
   *
   * @param keyId the key to use for re-encryption
   * @param notifyEmail the email address to notify when the re-encryption task is complete
   * @return the task id
   */
  private String submitTask(final String keyId, final String notifyEmail) {
    checkReEncryptionSupported();
    checkTaskNotSubmitted();
    checkKeyExists(keyId);
    TaskInfo scheduledTask = Optional.ofNullable(maybeScheduleReEncrypt(keyId, notifyEmail))
        .orElseThrow(() -> new RuntimeException("Failed to schedule re-encryption task"));
    String newKeyId = scheduledTask.getConfiguration().getString("keyId");
    String oldKeyId = encryptionKeyValidator.getActiveKeyId().orElse(null);
    eventManager.post(new ActiveKeyChangeEvent(newKeyId, oldKeyId, UserIdHelper.get()));
    return scheduledTask.getId();
  }

  private TaskInfo maybeScheduleReEncrypt(final String keyId, final String notifyEmail) {
    try {
      return cooperation.on(() -> scheduleReEncryptTask(keyId, notifyEmail))
          .checkFunction(this::getReEncryptTask)
          .cooperate("schedule_re-encryption");
    }
    catch (Exception e) {
      return null;
    }
  }

  private TaskInfo scheduleReEncryptTask(final String keyId, final String notifyEmail) {
    return getReEncryptTask().orElseGet(() -> {
      log.debug("Scheduling re-encrypt task");
      TaskConfiguration taskConfiguration =
          taskScheduler.createTaskConfigurationInstance(ReEncryptTaskDescriptor.TYPE_ID);
      taskConfiguration.setString("keyId", keyId);
      Optional.ofNullable(notifyEmail).ifPresent(taskConfiguration::setAlertEmail);
      return taskScheduler.submit(taskConfiguration);
    });
  }

  private Optional<TaskInfo> getReEncryptTask() {
    return taskScheduler.listsTasks()
        .stream()
        .filter(taskInfo -> taskInfo.getTypeId().equals(ReEncryptTaskDescriptor.TYPE_ID))
        .findFirst();
  }
}
