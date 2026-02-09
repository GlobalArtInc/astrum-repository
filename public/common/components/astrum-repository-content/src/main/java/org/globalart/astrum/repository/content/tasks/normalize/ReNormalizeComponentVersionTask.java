
package org.globalart.astrum.repository.content.tasks.normalize;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.TimeUnit;
import java.util.stream.Stream;

import org.globalart.astrum.common.QualifierUtil;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.logging.task.ProgressLogIntervalHelper;
import org.globalart.astrum.logging.task.TaskLogType;
import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.content.store.InternalIds;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.search.normalize.VersionNormalizerService;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.ParallelTaskSupport;
import org.globalart.astrum.scheduling.TaskInterruptedException;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.DISABLE_RENORMALIZE_VERSION_TASK;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.scheduling.CancelableHelper.checkCancellation;

/**
 * Repairs {@code normalized_version} rows written under an incompatible encoding for the formats whose
 * dedicated/shared normalizer was introduced by ASTRUM-53066, after that encoding change left already-populated
 * rows stuck on the old generic-expander encoding (see {@link ScheduleReNormalizationMigrationStep}).
 * <p>
 * Deliberately isolated from {@link NormalizeComponentVersionTask}: never reads/writes
 * {@code {format}.normalized.version.available}, never posts {@link FormatVersionNormalizedEvent}, so it cannot
 * re-trigger the existing null-only backfill migration.
 */
@org.springframework.stereotype.Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@TaskLogging(TaskLogType.TASK_LOG_ONLY_WITH_PROGRESS)
public class ReNormalizeComponentVersionTask
    extends ParallelTaskSupport
    implements Cancelable
{
  /**
   * Single source of truth for the formats affected by the ASTRUM-53066 encoding change, referenced by the
   * migration step, this task, and
   * {@link org.globalart.astrum.repository.content.internal.ReNormalizationTaskLauncher}.
   */
  public static final Set<String> RENORMALIZE_FORMATS = Set.of(
      "npm", "helm", "terraform", "swift", "pub",
      "nuget", "pypi", "apt", "go", "yum", "rubygems");

  /**
   * Shared with {@link ScheduleReNormalizationMigrationStep} (raises the gate) and
   * {@link org.globalart.astrum.repository.content.internal.ReNormalizationTaskLauncher} (checks whether the gate
   * is still raised after a restart).
   */
  public static final String GATE_KEY_FORMAT = "%s.renormalization.in_progress";

  static final String CHECKPOINT_KEY_FORMAT = "cleanup.renormalization.progress.%s.%s";

  /**
   * Fibonacci backoff (in seconds) between retries of a failed batch write, capped at {@link #BACKOFF_CAP_SECONDS}.
   */
  private static final long[] BACKOFF_FIBONACCI_SECONDS = {1, 1, 2, 3, 5, 8, 13, 21};

  private static final long BACKOFF_CAP_SECONDS = 30;

  private static final long CANCELLATION_POLL_INTERVAL_MILLIS = 100;

  /**
   * Shared with {@link org.globalart.astrum.repository.content.internal.ReNormalizationTaskLauncher}, which treats
   * the presence of this key as a sign that a repair run was interrupted before {@link #result()} could clear it.
   */
  public static final String CURSOR_KEY = "cleanup.renormalization.cursor";

  private final VersionNormalizerService versionNormalizerService;

  private final GlobalKeyValueStore globalKeyValueStore;

  private final RepositoryManager repositoryManager;

  private final Map<String, FormatStoreManager> formatStoreManagers;

  private final boolean disableTask;

  private final int maxWriteRetries;

  private ProgressLogIntervalHelper progressLogger;

  @Autowired
  public ReNormalizeComponentVersionTask(
      final VersionNormalizerService versionNormalizerService,
      final GlobalKeyValueStore globalKeyValueStore,
      final RepositoryManager repositoryManager,
      final List<FormatStoreManager> formatStoreManagersList,
      @Value("${" + DISABLE_RENORMALIZE_VERSION_TASK + ":false}") final boolean disableTask,
      @Value("${astrum.cleanup.renormalize.component.version.concurrencyLimit:5}") final int concurrencyLimit,
      @Value("${astrum.cleanup.renormalize.component.version.maxWriteRetries:5}") final int maxWriteRetries)
  {
    super(true, concurrencyLimit);
    this.versionNormalizerService = checkNotNull(versionNormalizerService);
    this.globalKeyValueStore = checkNotNull(globalKeyValueStore);
    this.repositoryManager = checkNotNull(repositoryManager);
    this.formatStoreManagers = QualifierUtil.buildQualifierBeanMap(checkNotNull(formatStoreManagersList));
    this.disableTask = disableTask;
    this.maxWriteRetries = maxWriteRetries;
  }

  @Override
  public String getMessage() {
    return "repair normalized_version column for formats affected by the version-normalizer encoding change";
  }

  @Override
  protected Stream<Runnable> jobStream(final ProgressLogIntervalHelper progress) {
    if (disableTask) {
      throw new TaskInterruptedException("The re-normalize version task was disabled", disableTask);
    }
    progressLogger = progress;

    // marks a repair run as in-progress; cleared in result() once every target format finishes cleanly, so an
    // interrupted run leaves this set as a resume signal for ReNormalizationTaskLauncher
    globalKeyValueStore.setBoolean(CURSOR_KEY, true);

    return RENORMALIZE_FORMATS.stream()
        .filter(formatStoreManagers::containsKey)
        // on a resumed run, formats that already finished have had their gate cleared - skip re-scanning them
        .filter(this::gateRaised)
        .map(format -> (Runnable) () -> renormalizeFormat(format));
  }

  boolean gateRaised(final String format) {
    return globalKeyValueStore.getKey(GATE_KEY_FORMAT.formatted(format))
        .map(AstrumKeyValue::getAsBoolean)
        .orElse(false);
  }

  void renormalizeFormat(final String format) {
    log.info("re-normalizing {} components version", format);

    ComponentStore<?> componentStore = formatStoreManagers.get(format).componentStore(DEFAULT_DATASTORE_NAME);

    for (int repositoryId : repositoryIdsForFormat(format)) {
      checkCancellation();
      renormalizeRepository(format, componentStore, repositoryId);
    }

    finishFormat(format);
    progressLogger.info("re-normalization complete for {} format - elapsed : {}", format, progressLogger.getElapsed());
  }

  void renormalizeRepository(
      final String format,
      final ComponentStore<?> componentStore,
      final int repositoryId)
  {
    String token = getCheckpoint(format, repositoryId);
    Continuation<Component> page =
        componentStore.browseComponents(repositoryId, Continuations.BROWSE_LIMIT, token, null, null, null);

    while (!page.isEmpty()) {
      // recomputed rows are written back in a single transaction per page rather than one transaction per
      // component - the latter meant up to Continuations.BROWSE_LIMIT round trips/commits per page
      List<ComponentData> toUpdate = new ArrayList<>(page.size());
      // a normalize failure is deterministic (bad version string) - propagates immediately rather than being
      // swallowed, so this format's gate is never cleared while a stale row is left behind
      page.forEach(component -> renormalizeComponent(component, format, repositoryId, toUpdate));
      if (!toUpdate.isEmpty()) {
        writePageWithRetry(format, componentStore, repositoryId, toUpdate);
      }

      String nextToken = page.nextContinuationToken();
      if (nextToken == null) {
        break;
      }

      // persist only after a successful page write, so an interruption or abort resumes exactly here, never
      // skipping a page that was never durably written
      setCheckpoint(format, repositoryId, nextToken);
      checkCancellation();

      page = componentStore.browseComponents(repositoryId, Continuations.BROWSE_LIMIT, nextToken, null, null, null);
    }

    clearCheckpoint(format, repositoryId);
  }

  void renormalizeComponent(
      final Component component,
      final String format,
      final int repositoryId,
      final List<ComponentData> toUpdate)
  {
    if (!(component instanceof ComponentData componentData)) {
      return;
    }

    try {
      // every row is recomputed unconditionally - the write is idempotent, and there is no reliable way to tell
      // an up-to-date encoding from a stale one without recomputing it
      componentData
          .setNormalizedVersion(versionNormalizerService.getNormalizedVersionByFormat(component.version(), format));
      toUpdate.add(componentData);
    }
    catch (Exception e) {
      // deterministic (bad version) - retrying won't help; stop this format so its gate stays raised and the
      // culprit remains visible instead of silently reopening the retain-N safety window from ASTRUM-54915
      log.error(
          "re-normalization stopped for format={}: component id={} repositoryId={} version='{}' could not be "
              + "normalized; retain-N stays DISABLED for {} until this is resolved (delete the component or fix "
              + "the normalizer). Will retry on next restart.",
          format, componentData.componentId(), repositoryId, component.version(), format, e);
      throw new RuntimeException("re-normalization stopped for format=" + format, e);
    }
  }

  /**
   * Retries a page's batch write with Fibonacci backoff, since a batch write failure is typically a transient
   * DB/connection issue unlike a normalize failure. The checkpoint for this page is only advanced by the caller
   * once this method returns, so retry exhaustion leaves the checkpoint at the last successfully-written page.
   */
  private void writePageWithRetry(
      final String format,
      final ComponentStore<?> componentStore,
      final int repositoryId,
      final List<ComponentData> toUpdate)
  {
    Exception lastFailure = null;
    for (int attempt = 1; attempt <= maxWriteRetries; attempt++) {
      try {
        componentStore.updateComponentsNormalizedVersion(toUpdate);
        return;
      }
      catch (Exception e) {
        lastFailure = e;
        log.warn("re-normalization batch write attempt {}/{} failed for format={} repositoryId={}: {}",
            attempt, maxWriteRetries, format, repositoryId, e.getMessage());
        if (attempt < maxWriteRetries) {
          sleepInterruptibly(backoffSeconds(attempt));
        }
      }
    }

    log.error(
        "re-normalization stopped for format={} repositoryId={}: batch write failed after {} attempts; checkpoint "
            + "left at last successful page, retain-N stays DISABLED for {} until it completes on a future restart.",
        format, repositoryId, maxWriteRetries, format, lastFailure);
    throw new RuntimeException("re-normalization stopped for format=" + format, lastFailure);
  }

  private static long backoffSeconds(final int attempt) {
    int index = Math.min(attempt - 1, BACKOFF_FIBONACCI_SECONDS.length - 1);
    return Math.min(BACKOFF_FIBONACCI_SECONDS[index], BACKOFF_CAP_SECONDS);
  }

  /**
   * Sleeps in short increments, checking cancellation between each one, so a shutdown/cancel is honored promptly
   * instead of being blocked behind a long backoff sleep.
   */
  private void sleepInterruptibly(final long seconds) {
    long remainingMillis = TimeUnit.SECONDS.toMillis(seconds);
    while (remainingMillis > 0) {
      checkCancellation();
      long chunk = Math.min(remainingMillis, CANCELLATION_POLL_INTERVAL_MILLIS);
      try {
        Thread.sleep(chunk);
      }
      catch (InterruptedException e) {
        Thread.currentThread().interrupt();
        checkCancellation();
        return;
      }
      remainingMillis -= chunk;
    }
  }

  List<Integer> repositoryIdsForFormat(final String format) {
    List<Integer> repositoryIds = new ArrayList<>();
    for (Repository repository : repositoryManager.browse()) {
      if (format.equals(repository.getFormat().getValue())) {
        InternalIds.contentRepositoryId(repository).ifPresent(repositoryIds::add);
      }
    }
    repositoryIds.sort(Comparator.naturalOrder());
    return repositoryIds;
  }

  String getCheckpoint(final String format, final int repositoryId) {
    return globalKeyValueStore.getKey(checkpointKey(format, repositoryId))
        .map(AstrumKeyValue::getAsString)
        .orElse(null);
  }

  void setCheckpoint(final String format, final int repositoryId, final String token) {
    globalKeyValueStore.setString(checkpointKey(format, repositoryId), token);
  }

  void clearCheckpoint(final String format, final int repositoryId) {
    globalKeyValueStore.removeKey(checkpointKey(format, repositoryId));
  }

  static String checkpointKey(final String format, final int repositoryId) {
    return CHECKPOINT_KEY_FORMAT.formatted(format, repositoryId);
  }

  void finishFormat(final String format) {
    log.info("re-normalization complete for {} format; clearing gate", format);
    // removing the key posts a KeyValueEvent, which ProCleanupFeatureCheck already reacts to by recomputing
    // retain-N support for this format - no separate event is needed here
    globalKeyValueStore.removeKey(GATE_KEY_FORMAT.formatted(format));
  }

  @Override
  protected Object result() {
    // only reached once every scheduled format has finished without throwing (ParallelTaskSupport propagates any
    // job exception before calling result()), so the whole repair run is complete
    globalKeyValueStore.removeKey(CURSOR_KEY);
    return null;
  }
}
