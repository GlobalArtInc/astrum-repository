
package org.globalart.astrum.self.hosted.blobstore.s3.upgrade;

import java.time.Duration;
import java.time.LocalDateTime;
import java.time.ZoneOffset;
import java.time.temporal.TemporalAmount;
import java.util.Date;
import java.util.List;
import java.util.Objects;
import java.util.stream.StreamSupport;

import javax.annotation.Nullable;

import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.blobstore.s3.S3BlobStoreConfigurationHelper;
import org.globalart.astrum.blobstore.s3.internal.S3BlobStore;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.TaskSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * This task creates compact blob store tasks for S3 blob stores based on the old configuration setting for expiration.
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class ScheduleS3CompactTasksTask
    extends TaskSupport
{
  private static final String BLOBSTORE_NAME = "blobstoreName";

  private static final String BLOBS_OLDER_THAN = "blobsOlderThan";

  private static final int DEFAULT_EXPIRATION_IN_DAYS = 3;

  private final BlobStoreManager blobstoreManager;

  private final TaskScheduler taskScheduler;

  @Autowired
  public ScheduleS3CompactTasksTask(
      final BlobStoreManager blobstoreManager,
      final TaskScheduler taskScheduler)
  {
    this.blobstoreManager = checkNotNull(blobstoreManager);
    this.taskScheduler = checkNotNull(taskScheduler);
  }

  @Override
  public String getMessage() {
    return "Migrating records of soft-deleted blobs";
  }

  @Override
  protected Object execute() throws Exception {
    List<TaskConfiguration> taskConfigurations = StreamSupport.stream(blobstoreManager.browse().spliterator(), false)
        .map(BlobStore::getBlobStoreConfiguration)
        .filter(configuration -> S3BlobStore.TYPE.equals(configuration.getType()))
        .map(this::createTaskConfiguration)
        .filter(Objects::nonNull)
        .toList();

    if (taskConfigurations.isEmpty()) {
      log.info("No blobstores configured with an expiration");
      return null;
    }

    TemporalAmount interval = Duration.ofMinutes(24 * 60 / taskConfigurations.size());
    LocalDateTime initial = LocalDateTime.now();

    for (TaskConfiguration taskConfiguration : taskConfigurations) {
      taskScheduler.scheduleTask(taskConfiguration,
          taskScheduler.getScheduleFactory().daily(new Date(initial.toInstant(ZoneOffset.UTC).toEpochMilli())));

      if (log.isInfoEnabled()) {
        log.info("Scheduled daily compact blobstore task for blobstore {} with expiration {}",
            taskConfiguration.getString(BLOBSTORE_NAME), taskConfiguration.getString(BLOBS_OLDER_THAN));
      }

      initial = initial.plus(interval);
    }

    log.info("Scheduled {} compact tasks for S3 blobstores", taskConfigurations.size());

    return taskConfigurations.size();
  }

  @Nullable
  private TaskConfiguration createTaskConfiguration(final BlobStoreConfiguration configuration) {
    String name = configuration.getName();
    int expiration = getExpiration(configuration);
    if (expiration < 0) {
      log.debug("Blobstore {} has a configured expiration of {}, skipping", name, expiration);
      return null;
    }

    TaskConfiguration task = taskScheduler.createTaskConfigurationInstance("blobstore.compact");
    task.setString(BLOBSTORE_NAME, name);
    task.setInteger(BLOBS_OLDER_THAN, expiration);
    return task;
  }

  private static int getExpiration(final BlobStoreConfiguration configuration) {
    return Integer.parseInt(configuration.attributes(S3BlobStoreConfigurationHelper.CONFIG_KEY)
        .get("expiration", DEFAULT_EXPIRATION_IN_DAYS)
        .toString());
  }
}
