package org.globalart.astrum.repository.content.downloads;

import java.time.Instant;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Collection;
import java.util.EnumMap;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.OptionalInt;
import java.util.Queue;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentLinkedQueue;
import java.util.concurrent.atomic.AtomicLong;
import javax.annotation.Nullable;

import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.scheduling.PeriodicJobService.PeriodicJob;
import org.globalart.astrum.common.stateguard.StateGuardLifecycleSupport;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.event.asset.AssetDeletedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetPurgedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentDeletedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentPurgedEvent;
import org.globalart.astrum.repository.content.event.repository.ContentRepositoryDeletedEvent;
import org.globalart.astrum.repository.content.store.ContentStoreEvent;
import org.globalart.astrum.repository.content.store.InternalIds;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.collect.Lists;
import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toCollection;
import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SERVICES;
import static org.globalart.astrum.repository.content.store.InternalIds.contentRepositoryId;
import static org.globalart.astrum.repository.content.store.InternalIds.internalAssetId;
import static org.globalart.astrum.repository.content.store.InternalIds.internalComponentId;

/**
 * Counts the downloads of every asset.
 *
 * Downloads are added up in memory, per format and internal asset id, and written to the {@link AssetDownloadCountStore}
 * every {@code astrum.downloads.flushSeconds} seconds and when the service stops. Reads add the downloads that are not
 * written yet, so a count changes as soon as an asset is downloaded. Rows of deleted assets, components and
 * repositories are deleted by the same periodic job, after the downloads it writes.
 *
 * Counting does not post asset events, so downloads neither reindex the asset nor wake up event subscribers.
 */
@ManagedLifecycle(phase = SERVICES)
@Component
public class AssetDownloadCounter
    extends StateGuardLifecycleSupport
    implements EventAware
{
  private static final String FLUSH_SECONDS_KEY = "astrum.downloads.flushSeconds";

  private final AssetDownloadCountStore store;

  private final PeriodicJobService periodicJobService;

  private final int flushSeconds;

  private final Map<Key, Pending> pending = new ConcurrentHashMap<>();

  private final Queue<Removal> removals = new ConcurrentLinkedQueue<>();

  private final Object flushMutex = new Object();

  private PeriodicJob flushJob;

  @Autowired
  public AssetDownloadCounter(
      final AssetDownloadCountStore store,
      final PeriodicJobService periodicJobService,
      @Value("${" + FLUSH_SECONDS_KEY + ":10}") final int flushSeconds)
  {
    this.store = checkNotNull(store);
    this.periodicJobService = checkNotNull(periodicJobService);
    checkArgument(flushSeconds > 0, FLUSH_SECONDS_KEY + " must be positive");
    this.flushSeconds = flushSeconds;
  }

  @Override
  protected void doStart() throws Exception {
    periodicJobService.startUsing();
    flushJob = periodicJobService.schedule(this::flushQuietly, flushSeconds);
  }

  @Override
  protected void doStop() throws Exception {
    flushJob.cancel();
    flushJob = null;
    periodicJobService.stopUsing();
    flushQuietly();
  }

  /**
   * Counts one download of the given asset of the given format.
   */
  public void recordDownload(final String format, final Asset asset) {
    OptionalInt componentId = internalComponentId(asset);
    recordDownload(format, internalAssetId(asset), contentRepositoryId(asset),
        componentId.isPresent() ? componentId.getAsInt() : null, System.currentTimeMillis());
  }

  @VisibleForTesting
  void recordDownload(
      final String format,
      final int assetId,
      final int repositoryId,
      @Nullable final Integer componentId,
      final long downloadedAt)
  {
    checkNotNull(format);
    pending.compute(new Key(format, assetId), (key, existing) -> {
      Pending downloads = existing != null ? existing : new Pending(repositoryId, componentId);
      downloads.add(1, downloadedAt);
      return downloads;
    });
  }

  /**
   * Returns how often the given asset of the given format was downloaded.
   */
  public long getDownloadCount(final String format, final Asset asset) {
    return getDownloadCounts(format, List.of(asset)).of(asset);
  }

  /**
   * Returns how often each of the given assets of the given format was downloaded, reading the store once per batch of
   * assets rather than once per asset.
   */
  public AssetDownloadCounts getDownloadCounts(final String format, final Collection<? extends Asset> assets) {
    Set<Integer> assetIds = assets.stream()
        .map(InternalIds::internalAssetId)
        .collect(toCollection(LinkedHashSet::new));
    return getDownloadCountsById(format, assetIds);
  }

  @VisibleForTesting
  AssetDownloadCounts getDownloadCountsById(final String format, final Collection<Integer> assetIds) {
    checkNotNull(format);
    Map<Integer, Long> counts = new HashMap<>();
    if (!assetIds.isEmpty()) {
      store.browse(format, assetIds)
          .forEach(row -> counts.merge(row.getAssetId(), row.getDownloadCount(), Long::sum));
      for (Integer assetId : assetIds) {
        Pending downloads = pending.get(new Key(format, assetId));
        if (downloads != null) {
          counts.merge(assetId, downloads.count(), Long::sum);
        }
      }
    }
    return new AssetDownloadCounts(counts);
  }

  /**
   * Writes the downloads counted so far, then deletes the rows of assets deleted so far.
   */
  public void flush() {
    synchronized (flushMutex) {
      writeDownloads(drainDownloads());
      applyRemovals();
    }
  }

  @VisibleForTesting
  int pendingCount() {
    return pending.size();
  }

  private void flushQuietly() {
    try {
      flush();
    }
    catch (Exception e) {
      log.warn("Failed to write asset download counts", e);
    }
  }

  private List<AssetDownloadCountData> drainDownloads() {
    List<AssetDownloadCountData> downloads = new ArrayList<>();
    for (Key key : pending.keySet()) {
      Pending removed = pending.remove(key);
      if (removed != null) {
        downloads.add(removed.toData(key));
      }
    }
    return downloads;
  }

  private void writeDownloads(final List<AssetDownloadCountData> downloads) {
    List<List<AssetDownloadCountData>> batches = Lists.partition(downloads, AssetDownloadCountStore.BATCH_SIZE);
    for (int i = 0; i < batches.size(); i++) {
      try {
        store.addDownloads(batches.get(i));
      }
      catch (RuntimeException e) {
        batches.subList(i, batches.size()).forEach(batch -> batch.forEach(this::restore));
        throw e;
      }
    }
    if (!downloads.isEmpty()) {
      log.debug("Wrote the download counts of {} assets", downloads.size());
    }
  }

  private void restore(final AssetDownloadCountData downloads) {
    long downloadedAt = downloads.getLastDownloaded().toInstant().toEpochMilli();
    pending.compute(new Key(downloads.getFormat(), downloads.getAssetId()), (key, existing) -> {
      Pending restored = existing != null ? existing
          : new Pending(downloads.getRepositoryId(), downloads.getComponentId());
      restored.add(downloads.getDownloadCount(), downloadedAt);
      return restored;
    });
  }

  private void applyRemovals() {
    List<Removal> drained = new ArrayList<>();
    Removal removal;
    while ((removal = removals.poll()) != null) {
      drained.add(removal);
    }
    if (drained.isEmpty()) {
      return;
    }
    Map<RemovalKind, Map<String, Set<Integer>>> grouped = new EnumMap<>(RemovalKind.class);
    drained.forEach(each -> grouped.computeIfAbsent(each.kind(), kind -> new HashMap<>())
        .computeIfAbsent(each.format(), format -> new LinkedHashSet<>())
        .add(each.id()));
    try {
      grouped.forEach((kind, idsByFormat) -> idsByFormat.forEach((format, ids) -> kind.delete(store, format, ids)));
    }
    catch (RuntimeException e) {
      removals.addAll(drained);
      throw e;
    }
    log.debug("Deleted the download counts of {} removed assets, components and repositories", drained.size());
  }

  @AllowConcurrentEvents
  @Subscribe
  public void on(final AssetDeletedEvent event) {
    String format = formatOf(event);
    if (format != null) {
      removeAssets(format, new int[]{internalAssetId(event.getAsset())});
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void on(final AssetPurgedEvent event) {
    String format = formatOf(event);
    if (format != null) {
      removeAssets(format, event.getAssetIds());
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void on(final ComponentDeletedEvent event) {
    String format = formatOf(event);
    if (format != null) {
      removeComponents(format, new int[]{internalComponentId(event.getComponent())});
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void on(final ComponentPurgedEvent event) {
    String format = formatOf(event);
    if (format != null) {
      removeComponents(format, event.getComponentIds());
    }
  }

  @AllowConcurrentEvents
  @Subscribe
  public void on(final ContentRepositoryDeletedEvent event) {
    String format = event.getFormat();
    int repositoryId = contentRepositoryId(event);
    pending.entrySet().removeIf(entry -> entry.getKey().format().equals(format)
        && entry.getValue().repositoryId == repositoryId);
    removals.add(new Removal(RemovalKind.REPOSITORY, format, repositoryId));
  }

  private void removeAssets(final String format, final int[] assetIds) {
    for (int assetId : assetIds) {
      pending.remove(new Key(format, assetId));
      removals.add(new Removal(RemovalKind.ASSET, format, assetId));
    }
  }

  private void removeComponents(final String format, final int[] componentIds) {
    Set<Integer> ids = Arrays.stream(componentIds).boxed().collect(toCollection(HashSet::new));
    pending.entrySet().removeIf(entry -> entry.getKey().format().equals(format)
        && ids.contains(entry.getValue().componentId));
    ids.forEach(componentId -> removals.add(new Removal(RemovalKind.COMPONENT, format, componentId)));
  }

  @Nullable
  private String formatOf(final ContentStoreEvent event) {
    String format = event.getFormat();
    if (format == null) {
      log.debug("Cannot tell the format of {}; its download counts stay", event);
    }
    return format;
  }

  private record Key(String format, int assetId)
  {
  }

  /**
   * Downloads of one asset that are not written yet. Only changed inside {@link Map#compute}, so that a flush removing
   * the entry sees every download added to it.
   */
  private static final class Pending
  {
    private final int repositoryId;

    @Nullable
    private final Integer componentId;

    private final AtomicLong count = new AtomicLong();

    private final AtomicLong lastDownloaded = new AtomicLong();

    private Pending(final int repositoryId, @Nullable final Integer componentId) {
      this.repositoryId = repositoryId;
      this.componentId = componentId;
    }

    private void add(final long downloads, final long downloadedAt) {
      count.addAndGet(downloads);
      lastDownloaded.accumulateAndGet(downloadedAt, Math::max);
    }

    private long count() {
      return count.get();
    }

    private AssetDownloadCountData toData(final Key key) {
      return new AssetDownloadCountData(key.format(), key.assetId(), repositoryId, componentId, count.get(),
          OffsetDateTime.ofInstant(Instant.ofEpochMilli(lastDownloaded.get()), ZoneOffset.UTC));
    }
  }

  /**
   * An asset, component or content repository whose download counts are to be deleted.
   */
  private record Removal(RemovalKind kind, String format, int id)
  {
  }

  private enum RemovalKind
  {
    ASSET
        {
          @Override
          void delete(final AssetDownloadCountStore store, final String format, final Set<Integer> ids) {
            store.deleteAssets(format, ids);
          }
        },
    COMPONENT
        {
          @Override
          void delete(final AssetDownloadCountStore store, final String format, final Set<Integer> ids) {
            store.deleteComponents(format, ids);
          }
        },
    REPOSITORY
        {
          @Override
          void delete(final AssetDownloadCountStore store, final String format, final Set<Integer> ids) {
            ids.forEach(repositoryId -> store.deleteRepository(format, repositoryId));
          }
        };

    abstract void delete(AssetDownloadCountStore store, String format, Set<Integer> ids);
  }
}
