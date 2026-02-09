
package org.globalart.astrum.repository.content.store.internal;

import java.util.Date;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.lifecycle.LifecycleSupport;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.repository.RepositoryStartedEvent;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskInfo;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.events.TaskDeletedEvent;
import org.globalart.astrum.scheduling.schedule.Schedule;

import com.google.common.collect.HashMultimap;
import com.google.common.collect.ImmutableMap;
import com.google.common.collect.SetMultimap;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.collect.Multimaps.synchronizedSetMultimap;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.DATA_STORE_NAME;
import static org.globalart.astrum.repository.config.ConfigurationConstants.STORAGE;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTask.CRON_SCHEDULE;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.CONTENT_STORE_FIELD_ID;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.FORMAT_FIELD_ID;
import static org.globalart.astrum.repository.content.store.internal.AssetBlobCleanupTaskDescriptor.TYPE_ID;
import org.springframework.stereotype.Component;

/**
 * @since 3.24
 */
@Component
@ManagedLifecycle(phase = TASKS)
public class AssetBlobCleanupTaskManager
    extends LifecycleSupport
    implements EventAware, EventAware.Asynchronous
{
  private final TaskScheduler taskScheduler;

  private final SetMultimap<String, String> activeFormatStores = synchronizedSetMultimap(HashMultimap.create());

  @Autowired
  public AssetBlobCleanupTaskManager(final TaskScheduler taskScheduler) {
    this.taskScheduler = checkNotNull(taskScheduler);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final RepositoryStartedEvent event) {
    String format = event.getRepository().getFormat().getValue();

    Configuration repositoryConfiguration = event.getRepository().getConfiguration();
    NestedAttributesMap storageAttributes = repositoryConfiguration.attributes(STORAGE);
    String contentStore = (String) storageAttributes.get(DATA_STORE_NAME, DEFAULT_DATASTORE_NAME);

    if (activeFormatStores.put(format, contentStore) && isStarted()) {
      scheduleAssetBlobCleanupTask(format, contentStore);
    }
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final TaskDeletedEvent event) {
    TaskInfo taskInfo = event.getTaskInfo();
    if (TYPE_ID.equals(taskInfo.getTypeId())) {
      TaskConfiguration taskConfiguration = taskInfo.getConfiguration();
      String format = taskConfiguration.getString(FORMAT_FIELD_ID);
      String contentStore = taskConfiguration.getString(CONTENT_STORE_FIELD_ID);
      activeFormatStores.remove(format, contentStore);
    }
  }

  @Override
  protected void doStart() throws Exception {
    activeFormatStores.forEach(this::scheduleAssetBlobCleanupTask);
  }

  private void scheduleAssetBlobCleanupTask(final String format, final String contentStore) {
    Map<String, String> settings = ImmutableMap.of(FORMAT_FIELD_ID, format, CONTENT_STORE_FIELD_ID, contentStore);
    if (taskScheduler.getTaskByTypeId(TYPE_ID, settings) == null) {
      TaskConfiguration taskConfiguration = taskScheduler.createTaskConfigurationInstance(TYPE_ID);
      taskConfiguration.setName("Cleanup unused " + format + " blobs from " + contentStore);
      taskConfiguration.setString(FORMAT_FIELD_ID, format);
      taskConfiguration.setString(CONTENT_STORE_FIELD_ID, contentStore);
      Schedule schedule = taskScheduler.getScheduleFactory().cron(new Date(), CRON_SCHEDULE);
      log.info("Scheduling cleanup of unused {} blobs from {}", format, contentStore);
      taskScheduler.scheduleTask(taskConfiguration, schedule);
    }
  }
}
