
package org.globalart.astrum.repository.content.browse;

import java.util.concurrent.TimeUnit;
import java.util.stream.Collectors;
import java.util.stream.StreamSupport;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.browse.node.RebuildBrowseNodesTaskDescriptor;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.scheduling.TaskConfiguration;
import org.globalart.astrum.scheduling.TaskScheduler;
import org.globalart.astrum.upgrade.datastore.UpgradeContext;

import com.google.common.base.Stopwatch;
import com.google.common.collect.ImmutableMap;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.TASKS;
import org.springframework.stereotype.Component;

/**
 * Triggers a rebuild task for all repositories when requested. This is largely for upgrades or other early
 * phase code before services have started.
 *
 * @since 3.33
 */
@Component
@ManagedLifecycle(phase = TASKS)
public class RebuildBrowseNodesManager
    extends StateGuardLifecycleSupport
{
  /**
   * {@link UpgradeContext} flag key raised by an UPGRADE-phase migration step to request a one-time
   * browse-node rebuild when this manager starts (TASKS phase).
   */
  public static final String REBUILD_BROWSE_NODES_ON_START = "browse.rebuildNodesOnStart";

  private static final String ALL_REPOSITORIES = "*";

  private final TaskScheduler taskScheduler;

  private final RepositoryManager repositoryManager;

  private final PeriodicJobService periodicJobService;

  private final UpgradeContext upgradeContext;

  @Autowired
  public RebuildBrowseNodesManager(
      final TaskScheduler taskScheduler,
      final RepositoryManager repositoryManager,
      final PeriodicJobService periodicJobService,
      final UpgradeContext upgradeContext)
  {
    this.taskScheduler = checkNotNull(taskScheduler);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.periodicJobService = checkNotNull(periodicJobService);
    this.upgradeContext = checkNotNull(upgradeContext);
  }

  @Override
  protected void doStart() { // NOSONAR
    if (!upgradeContext.isFlagSet(REBUILD_BROWSE_NODES_ON_START)) {
      return;
    }

    periodicJobService.runOnce(this::maybeRebuild, 0);
  }

  private void maybeRebuild() {
    Stopwatch sw = Stopwatch.createStarted();
    try {
      String repositoryNames = StreamSupport.stream(repositoryManager.browse().spliterator(), false)
          .filter(this::hasAssets)
          .map(Repository::getName)
          .collect(Collectors.joining(","));

      if (!Strings2.isEmpty(repositoryNames)) {
        boolean existingTask = taskScheduler.findWaitingTask(RebuildBrowseNodesTaskDescriptor.TYPE_ID,
            ImmutableMap.of(RebuildBrowseNodesTaskDescriptor.REPOSITORY_NAME_FIELD_ID, ALL_REPOSITORIES));
        if (!existingTask) {
          launchNewTask(repositoryNames);
        }
      }
    }
    catch (Exception e) {
      log.error("Failed to determine if the browse nodes need to be rebuilt for any repositories", e);
    }
    log.debug("scheduling rebuild browse nodes tasks took {} ms", sw.elapsed(TimeUnit.MILLISECONDS));
  }

  private boolean hasAssets(final Repository repository) {
    return repository.facet(ContentFacet.class).assets().count() > 0;
  }

  private void launchNewTask(final String repositoryNames) {
    TaskConfiguration configuration = taskScheduler
        .createTaskConfigurationInstance(RebuildBrowseNodesTaskDescriptor.TYPE_ID);
    configuration.setString(RebuildBrowseNodesTaskDescriptor.REPOSITORY_NAME_FIELD_ID, repositoryNames);
    configuration.setName("Rebuild repository browse tree - (" + repositoryNames + ")");
    taskScheduler.submit(configuration);
  }
}
