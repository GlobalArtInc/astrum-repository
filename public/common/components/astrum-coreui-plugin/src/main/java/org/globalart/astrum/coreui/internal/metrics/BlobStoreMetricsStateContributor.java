
package org.globalart.astrum.coreui.internal.metrics;

import java.util.Map;
import java.util.Set;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.rapture.StateContributor;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.scheduling.TaskState;
import org.globalart.astrum.scheduling.internal.upgrade.datastore.UpgradeTaskStore;

import com.google.common.collect.ImmutableMap;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static java.util.Objects.requireNonNull;

/**
 * State contributor to indicate if blob store metrics-related upgrade tasks are currently being calculated.
 * This allows the UI to display "Calculating..." instead of "0" for metrics during database migration.
 *
 * @since 3.87
 */
@Component
public class BlobStoreMetricsStateContributor
    implements StateContributor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String BLOBSTORE_METRICS_CALCULATING = "astrum.datastore.blobstore.metrics.calculating";

  private static final Set<String> METRIC_TASK_IDS = Set.of(
      "astrum.blobstore.metrics.migration.task",
      "component.normalize.version");

  private static final String RECALCULATE_TASK_TYPE_ID = "blobstore.metrics.reconcile";

  private final UpgradeTaskStore upgradeTaskStore;

  private final TaskScheduler taskScheduler;

  @Autowired
  public BlobStoreMetricsStateContributor(
      final UpgradeTaskStore upgradeTaskStore,
      final TaskScheduler taskScheduler)
  {
    this.upgradeTaskStore = requireNonNull(upgradeTaskStore);
    this.taskScheduler = requireNonNull(taskScheduler);
  }

  @Override
  public Map<String, Object> getState() {
    boolean isCalculating = isUpgradeTaskRunning() || isRecalculateTaskRunning();

    log.debug("Blob store metrics calculating status: {}", isCalculating);

    return ImmutableMap.of(BLOBSTORE_METRICS_CALCULATING, isCalculating);
  }

  private boolean isUpgradeTaskRunning() {
    return upgradeTaskStore.browse()
        .anyMatch(task -> METRIC_TASK_IDS.contains(task.getTaskId()));
  }

  private boolean isRecalculateTaskRunning() {
    return taskScheduler.listsTasks()
        .stream()
        .filter(task -> RECALCULATE_TASK_TYPE_ID.equals(task.getTypeId()))
        .anyMatch(task -> task.getCurrentState().getState() == TaskState.RUNNING);
  }
}
