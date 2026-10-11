package org.globalart.astrum.repository.content.downloads;

import java.time.Instant;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.stream.Collectors;

import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.scheduling.PeriodicJobService.PeriodicJob;
import org.globalart.astrum.repository.content.event.asset.AssetDeletedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetPurgedEvent;
import org.globalart.astrum.repository.content.event.component.ComponentPurgedEvent;
import org.globalart.astrum.repository.content.event.repository.ContentRepositoryDeletedEvent;
import org.globalart.astrum.repository.content.store.AssetData;
import org.globalart.astrum.repository.content.store.ComponentData;
import org.globalart.astrum.repository.content.store.ContentRepositoryData;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.InOrder;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertThrows;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyCollection;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.inOrder;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AssetDownloadCounterTest
{
  private static final long NOW = Instant.parse("2026-10-01T12:00:00Z").toEpochMilli();

  @Mock
  private AssetDownloadCountStore store;

  @Mock
  private PeriodicJobService periodicJobService;

  @Mock
  private PeriodicJob periodicJob;

  private final List<AssetDownloadCountData> written = new ArrayList<>();

  private AssetDownloadCounter underTest;

  @Before
  public void setUp() {
    when(periodicJobService.schedule(any(Runnable.class), anyInt())).thenReturn(periodicJob);
    doAnswerAddDownloads();
    underTest = new AssetDownloadCounter(store, periodicJobService, 10);
  }

  @Test
  public void addsUpTheDownloadsOfEachAsset() {
    underTest.recordDownload("raw", 1, 10, 100, NOW);
    underTest.recordDownload("raw", 1, 10, 100, NOW + 2000);
    underTest.recordDownload("raw", 1, 10, 100, NOW + 1000);
    underTest.recordDownload("raw", 2, 10, null, NOW);
    underTest.recordDownload("maven2", 1, 20, null, NOW);

    underTest.flush();

    Map<String, AssetDownloadCountData> rows = writtenByKey();
    assertThat(rows.keySet(), containsInAnyOrder("raw:1", "raw:2", "maven2:1"));
    assertThat(rows.get("raw:1").getDownloadCount(), is(3L));
    assertThat(rows.get("raw:1").getComponentId(), is(100));
    assertThat(rows.get("raw:1").getRepositoryId(), is(10));
    assertThat(rows.get("raw:1").getLastDownloaded().toInstant().toEpochMilli(), is(NOW + 2000));
    assertThat(rows.get("raw:2").getDownloadCount(), is(1L));
    assertThat(rows.get("maven2:1").getRepositoryId(), is(20));
  }

  @Test
  public void recordsTheIdsOfTheAsset() {
    underTest.recordDownload("raw", asset(7, 3, 42));

    underTest.flush();

    AssetDownloadCountData row = writtenByKey().get("raw:7");
    assertThat(row.getDownloadCount(), is(1L));
    assertThat(row.getRepositoryId(), is(3));
    assertThat(row.getComponentId(), is(42));
  }

  @Test
  public void writesEachDownloadOnce() {
    underTest.recordDownload("raw", 1, 10, null, NOW);
    underTest.flush();
    underTest.flush();
    underTest.recordDownload("raw", 1, 10, null, NOW);
    underTest.flush();

    assertThat(written.stream().mapToLong(AssetDownloadCountData::getDownloadCount).sum(), is(2L));
    assertThat(underTest.pendingCount(), is(0));
  }

  @Test
  public void flushWithoutDownloadsWritesNothing() {
    underTest.flush();

    verify(store, never()).addDownloads(any());
  }

  @Test
  public void keepsTheDownloadsWhenWritingFails() {
    underTest.recordDownload("raw", 1, 10, null, NOW);
    underTest.recordDownload("raw", 1, 10, null, NOW);
    doThrow(new IllegalStateException("database down")).when(store).addDownloads(any());

    assertThrows(IllegalStateException.class, () -> underTest.flush());
    assertThat(underTest.pendingCount(), is(1));

    doAnswerAddDownloads();
    underTest.recordDownload("raw", 1, 10, null, NOW);
    underTest.flush();

    assertThat(writtenByKey().get("raw:1").getDownloadCount(), is(3L));
  }

  @Test
  public void readsAddTheDownloadsNotWrittenYet() {
    when(store.browse(eq("raw"), anyCollection()))
        .thenReturn(List.of(new AssetDownloadCountData("raw", 1, 10, null, 10, null)));
    underTest.recordDownload("raw", 1, 10, null, NOW);
    underTest.recordDownload("raw", 1, 10, null, NOW);
    underTest.recordDownload("raw", 2, 10, null, NOW);
    underTest.recordDownload("maven2", 3, 10, null, NOW);

    AssetDownloadCounts counts = underTest.getDownloadCounts("raw", List.of(asset(1, 10, null), asset(2, 10, null),
        asset(3, 10, null)));

    assertThat(counts.of(asset(1, 10, null)), is(12L));
    assertThat(counts.of(asset(2, 10, null)), is(1L));
    assertThat(counts.of(asset(3, 10, null)), is(0L));
    assertThat(counts.total(), is(13L));
  }

  @Test
  public void readsTheStoreOnceForAllAssets() {
    underTest.getDownloadCounts("raw", List.of(asset(1, 10, null), asset(2, 10, null), asset(2, 10, null)));

    ArgumentCaptor<Collection<Integer>> assetIds = ArgumentCaptor.forClass(Collection.class);
    verify(store, times(1)).browse(eq("raw"), assetIds.capture());
    assertThat(assetIds.getValue(), containsInAnyOrder(1, 2));
  }

  @Test
  public void readingNoAssetsDoesNotQueryTheStore() {
    assertThat(underTest.getDownloadCounts("raw", List.of()).total(), is(0L));

    verify(store, never()).browse(anyString(), anyCollection());
  }

  @Test
  public void countsDownloadsMadeConcurrently() throws Exception {
    int threads = 8;
    int downloadsPerThread = 2_000;
    ExecutorService executor = Executors.newFixedThreadPool(threads + 1);
    CountDownLatch start = new CountDownLatch(1);
    try {
      List<Future<?>> futures = new ArrayList<>();
      for (int i = 0; i < threads; i++) {
        futures.add(executor.submit(() -> {
          start.await();
          for (int j = 0; j < downloadsPerThread; j++) {
            underTest.recordDownload("raw", j % 3, 10, null, NOW);
          }
          return null;
        }));
      }
      futures.add(executor.submit(() -> {
        start.await();
        for (int j = 0; j < 200; j++) {
          underTest.flush();
        }
        return null;
      }));
      start.countDown();
      for (Future<?> future : futures) {
        future.get(30, TimeUnit.SECONDS);
      }
    }
    finally {
      executor.shutdownNow();
    }
    underTest.flush();

    assertThat(written.stream().mapToLong(AssetDownloadCountData::getDownloadCount).sum(),
        is((long) threads * downloadsPerThread));
  }

  @Test
  public void deletedAssetsLoseTheirPendingDownloadsAndRows() {
    underTest.recordDownload("raw", 1, 10, null, NOW);
    underTest.recordDownload("raw", 2, 10, null, NOW);
    AssetDeletedEvent event = mock(AssetDeletedEvent.class);
    when(event.getFormat()).thenReturn("raw");
    when(event.getAsset()).thenReturn(asset(1, 10, null));

    underTest.on(event);
    underTest.flush();

    assertThat(writtenByKey().keySet(), containsInAnyOrder("raw:2"));
    verify(store).deleteAssets(eq("raw"), eq(Set.of(1)));
  }

  @Test
  public void purgedAssetsAreDeletedTogetherAfterTheDownloadsAreWritten() {
    underTest.recordDownload("raw", 1, 10, null, NOW);
    underTest.recordDownload("raw", 9, 10, null, NOW);
    AssetPurgedEvent first = mock(AssetPurgedEvent.class);
    when(first.getFormat()).thenReturn("raw");
    when(first.getAssetIds()).thenReturn(new int[]{1, 2});
    AssetPurgedEvent second = mock(AssetPurgedEvent.class);
    when(second.getFormat()).thenReturn("raw");
    when(second.getAssetIds()).thenReturn(new int[]{3});

    underTest.on(first);
    underTest.on(second);
    underTest.flush();

    InOrder inOrder = inOrder(store);
    inOrder.verify(store).addDownloads(any());
    inOrder.verify(store).deleteAssets(eq("raw"), eq(Set.of(1, 2, 3)));
    assertThat(writtenByKey().keySet(), containsInAnyOrder("raw:9"));
  }

  @Test
  public void purgedComponentsLoseTheDownloadsOfTheirAssets() {
    underTest.recordDownload("raw", 1, 10, 100, NOW);
    underTest.recordDownload("raw", 2, 10, 200, NOW);
    underTest.recordDownload("maven2", 3, 10, 100, NOW);
    ComponentPurgedEvent event = mock(ComponentPurgedEvent.class);
    when(event.getFormat()).thenReturn("raw");
    when(event.getComponentIds()).thenReturn(new int[]{100});

    underTest.on(event);
    underTest.flush();

    assertThat(writtenByKey().keySet(), containsInAnyOrder("raw:2", "maven2:3"));
    verify(store).deleteComponents(eq("raw"), eq(Set.of(100)));
  }

  @Test
  public void deletedRepositoriesLoseTheDownloadsOfTheirAssets() {
    underTest.recordDownload("raw", 1, 10, null, NOW);
    underTest.recordDownload("raw", 2, 11, null, NOW);
    ContentRepositoryData contentRepository = new ContentRepositoryData();
    contentRepository.setRepositoryId(10);

    underTest.on(new ContentRepositoryDeletedEvent(contentRepository, "raw"));
    underTest.flush();

    assertThat(writtenByKey().keySet(), containsInAnyOrder("raw:2"));
    verify(store).deleteRepository("raw", 10);
  }

  @Test
  public void eventsOfUnknownFormatChangeNothing() {
    underTest.recordDownload("raw", 1, 10, null, NOW);
    AssetPurgedEvent event = mock(AssetPurgedEvent.class);
    when(event.getAssetIds()).thenReturn(new int[]{1});

    underTest.on(event);
    underTest.flush();

    assertThat(writtenByKey().keySet(), containsInAnyOrder("raw:1"));
    verify(store, never()).deleteAssets(anyString(), any());
  }

  @Test
  public void retriesDeletionsThatFailed() {
    AssetPurgedEvent event = mock(AssetPurgedEvent.class);
    when(event.getFormat()).thenReturn("raw");
    when(event.getAssetIds()).thenReturn(new int[]{1});
    underTest.on(event);
    when(store.deleteAssets(anyString(), any())).thenThrow(new IllegalStateException("database down")).thenReturn(1);

    assertThrows(IllegalStateException.class, () -> underTest.flush());
    underTest.flush();
    underTest.flush();

    verify(store, times(2)).deleteAssets(eq("raw"), eq(Set.of(1)));
  }

  @Test
  public void flushesPeriodicallyAndWhenStopped() throws Exception {
    underTest.start();
    ArgumentCaptor<Runnable> job = ArgumentCaptor.forClass(Runnable.class);
    verify(periodicJobService).startUsing();
    verify(periodicJobService).schedule(job.capture(), eq(10));

    underTest.recordDownload("raw", 1, 10, null, NOW);
    job.getValue().run();
    assertThat(writtenByKey().get("raw:1").getDownloadCount(), is(1L));

    underTest.recordDownload("raw", 2, 10, null, NOW);
    underTest.stop();

    verify(periodicJob).cancel();
    verify(periodicJobService).stopUsing();
    assertThat(writtenByKey().keySet(), containsInAnyOrder("raw:1", "raw:2"));
  }

  @Test
  public void periodicFlushSurvivesFailures() throws Exception {
    underTest.start();
    ArgumentCaptor<Runnable> job = ArgumentCaptor.forClass(Runnable.class);
    verify(periodicJobService).schedule(job.capture(), eq(10));
    underTest.recordDownload("raw", 1, 10, null, NOW);
    doThrow(new IllegalStateException("database down")).when(store).addDownloads(any());

    job.getValue().run();

    assertThat(underTest.pendingCount(), is(1));
  }

  @Test
  public void rejectsANonPositiveFlushPeriod() {
    assertThrows(IllegalArgumentException.class, () -> new AssetDownloadCounter(store, periodicJobService, 0));
  }

  private void doAnswerAddDownloads() {
    doAnswer(invocation -> {
      written.addAll(invocation.getArgument(0));
      return null;
    }).when(store).addDownloads(any());
  }

  private Map<String, AssetDownloadCountData> writtenByKey() {
    return written.stream().collect(Collectors.toMap(row -> row.getFormat() + ':' + row.getAssetId(), row -> row,
        (first, second) -> new AssetDownloadCountData(first.getFormat(), first.getAssetId(), first.getRepositoryId(),
            first.getComponentId(), first.getDownloadCount() + second.getDownloadCount(),
            second.getLastDownloaded())));
  }

  private static AssetData asset(final int assetId, final int repositoryId, final Integer componentId) {
    AssetData asset = new AssetData();
    asset.setAssetId(assetId);
    asset.setRepositoryId(repositoryId);
    if (componentId != null) {
      ComponentData component = new ComponentData();
      component.setComponentId(componentId);
      asset.setComponent(component);
    }
    return asset;
  }
}
