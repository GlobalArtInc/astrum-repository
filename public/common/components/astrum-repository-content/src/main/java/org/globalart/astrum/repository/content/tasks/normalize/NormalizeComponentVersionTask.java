
package org.globalart.astrum.repository.content.tasks.normalize;

import java.util.Map;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Stream;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.kv.GlobalKeyValueStore;
import org.globalart.astrum.kv.AstrumKeyValue;
import org.globalart.astrum.kv.ValueType;
import org.globalart.astrum.logging.task.ProgressLogIntervalHelper;
import org.globalart.astrum.logging.task.TaskLogType;
import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.ComponentStore;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.search.normalize.VersionNormalizerService;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.ParallelTaskSupport;
import org.globalart.astrum.scheduling.TaskInterruptedException;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.DISABLE_NORMALIZE_VERSION_TASK;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * System task to populate the {format}_component tables
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@TaskLogging(TaskLogType.TASK_LOG_ONLY_WITH_PROGRESS)
public class NormalizeComponentVersionTask
    extends ParallelTaskSupport
    implements Cancelable
{
  public static final String KEY_FORMAT = "%s.normalized.version.available";

  private final NormalizationPriorityService normalizationPriorityService;

  private final VersionNormalizerService versionNormalizerService;

  private final GlobalKeyValueStore globalKeyValueStore;

  private final EventManager eventManager;

  private final boolean disableTask;

  private ProgressLogIntervalHelper progressLogger;

  @Autowired
  public NormalizeComponentVersionTask(
      final NormalizationPriorityService priorityService,
      final VersionNormalizerService versionNormalizerService,
      final GlobalKeyValueStore globalKeyValueStore,
      final EventManager eventManager,
      @Value("${" + DISABLE_NORMALIZE_VERSION_TASK + ":false}") final boolean disableTask,
      @Value("${astrum.normalize.component.version.concurrencyLimit:5}") final int concurrencyLimit)
  {
    super(true, concurrencyLimit);
    this.normalizationPriorityService = checkNotNull(priorityService);
    this.versionNormalizerService = checkNotNull(versionNormalizerService);
    this.globalKeyValueStore = checkNotNull(globalKeyValueStore);
    this.eventManager = checkNotNull(eventManager);
    this.disableTask = disableTask;
  }

  @Override
  public String getMessage() {
    return "populate normalized_version column on {format}_component tables";
  }

  @Override
  protected Stream<Runnable> jobStream(final ProgressLogIntervalHelper progress) {
    if (disableTask) {
      throw new TaskInterruptedException("The normalize version task was disabled", disableTask);
    }
    progressLogger = progress;

    Map<String, FormatStoreManager> formats = normalizationPriorityService.getPrioritizedFormats();

    int totalCount = formats.size();
    AtomicInteger skippedCount = new AtomicInteger();
    AtomicInteger processedCount = new AtomicInteger();

    return formats.entrySet()
        .stream()
        .map(entry -> () -> processFormat(totalCount, skippedCount, processedCount, entry.getKey(), entry.getValue()));
  }

  private void processFormat(
      final int totalCount,
      final AtomicInteger skippedCount,
      final AtomicInteger processedCount,
      final String format,
      final FormatStoreManager manager)
  {
    log.info("normalizing {} components version", format);

    ComponentStore<?> componentStore = manager.componentStore(DEFAULT_DATASTORE_NAME);

    if (!isFormatNormalized(format, componentStore)) {
      // initially set normalization state as false
      setNormalizationState(format, false);
      normalizeFormat(format, componentStore);
      // once normalization is done set state as true
      setNormalizationState(format, true);
      // publish an event to let interested know the format has been normalized
      eventManager.post(new FormatVersionNormalizedEvent(format));

      int currentCount = processedCount.incrementAndGet();

      progressLogger.info(" task progress : {}% ({} of {} formats - skipped : {}) - elapsed : {}",
          Math.round((float) currentCount / totalCount * 100),
          currentCount, totalCount, skippedCount.get(), progressLogger.getElapsed());
    }
    else {
      log.debug("skipping {} format since is already normalized.", format);
      skippedCount.getAndIncrement();
    }
  }

  /**
   * Gets a normalization state for the given format
   *
   * @param format the format to perform the query
   * @return {@link Boolean} flag indicating the normalization state
   */
  private Boolean isFormatNormalized(final String format, final ComponentStore<?> componentStore) {
    return globalKeyValueStore.getKey(getFormatKey(format))
        .map(AstrumKeyValue::getAsBoolean)
        .orElseGet(() -> {
          log.debug("no previous normalization state for {} format", format);
          return false;
        })
        && componentStore.browseUnnormalized(1, null).isEmpty();
  }

  /**
   * Sets a normalization state for the given format
   *
   * @param format the format to set the as part of the key
   * @param value a {@link Boolean} flag indicating if the normalized version is available
   */
  private void setNormalizationState(final String format, final boolean value) {
    AstrumKeyValue kv = new AstrumKeyValue();
    kv.setKey(getFormatKey(format));
    kv.setType(ValueType.BOOLEAN);
    kv.setValue(value);

    globalKeyValueStore.setKey(kv);
  }

  /**
   * Builds a string key with the given format
   *
   * @param format the format to set as part of the key
   * @return a {@link String} value with the key
   */
  private static String getFormatKey(final String format) {
    return KEY_FORMAT.formatted(format);
  }

  /**
   * Normalizes version of {format}_component 's records
   *
   * @param format the given format
   * @param componentStore the format component store
   */
  private void normalizeFormat(final String format, final ComponentStore<?> componentStore) {
    Continuation<ComponentData> page =
        componentStore.browseUnnormalized(Continuations.BROWSE_LIMIT, null);

    int totalCount = componentStore.countUnnormalized();
    int processedCount = 0;

    log.info("found {} unnormalized records on {} components", totalCount, format);

    // Avoid an almost impossible divide by zero
    totalCount = Math.max(1, totalCount);

    while (!page.isEmpty()) {
      page.forEach((component) -> {
        String normalizedVersion = versionNormalizerService.getNormalizedVersionByFormat(component.version(), format);
        component.setNormalizedVersion(normalizedVersion);
        componentStore.updateComponentNormalizedVersion(component);
      });

      processedCount += page.size();

      log.info(" {} format progress : {}% ({} of {}) - elapsed : {}", format,
          Math.round((float) processedCount / totalCount * 100),
          processedCount, totalCount, progressLogger.getElapsed());

      String token = page.nextContinuationToken();
      if (token == null) {
        break;
      }

      page = componentStore.browseUnnormalized(Continuations.BROWSE_LIMIT, token);
    }
  }

  @Override
  protected Object result() {
    return null;
  }
}
