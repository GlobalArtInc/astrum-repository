package org.globalart.astrum.repository.content.downloads;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.IntStream;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

public class AssetDownloadCountStoreTest
{
  private static final OffsetDateTime EARLIER = OffsetDateTime.of(2026, 1, 1, 10, 0, 0, 0, ZoneOffset.UTC);

  private static final OffsetDateTime LATER = EARLIER.plusHours(5);

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME).access(AssetDownloadCountDAO.class);

  private AssetDownloadCountStore underTest;

  @Before
  public void setUp() {
    underTest = new AssetDownloadCountStore((DataSessionSupplier) sessionRule);
    removeRowsOfEarlierTests();
  }

  /**
   * In-memory H2 starts empty for every test, a PostgreSQL given by {@code test.jdbcUrl} keeps the rows.
   */
  private void removeRowsOfEarlierTests() {
    for (String format : List.of("raw", "maven2")) {
      underTest.deleteRepository(format, 10);
      underTest.deleteRepository(format, 11);
    }
  }

  @Test(expected = Test.None.class)
  public void canCreateTheSchemaMoreThanOnce() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      session.access(AssetDownloadCountDAO.class).createSchema();
      session.access(AssetDownloadCountDAO.class).createSchema();
    }
  }

  @Test
  public void createsARowForTheFirstDownloads() {
    underTest.addDownloads(List.of(row("raw", 1, 10, 100, 3, EARLIER)));

    AssetDownloadCountData stored = read("raw", 1);
    assertThat(stored.getDownloadCount(), is(3L));
    assertThat(stored.getRepositoryId(), is(10));
    assertThat(stored.getComponentId(), is(100));
    assertThat(stored.getLastDownloaded().isEqual(EARLIER), is(true));
  }

  @Test
  public void addsLaterDownloadsToTheStoredCount() {
    underTest.addDownloads(List.of(row("raw", 1, 10, 100, 3, LATER)));
    underTest.addDownloads(List.of(row("raw", 1, 10, null, 2, EARLIER)));
    underTest.addDownloads(List.of(row("raw", 1, 10, null, 4, EARLIER)));

    AssetDownloadCountData stored = read("raw", 1);
    assertThat(stored.getDownloadCount(), is(9L));
    assertThat(stored.getLastDownloaded().isEqual(LATER), is(true));
    assertThat(stored.getComponentId(), is(100));
  }

  @Test
  public void movesTheLastDownloadedTimeForward() {
    underTest.addDownloads(List.of(row("raw", 1, 10, null, 1, EARLIER)));
    underTest.addDownloads(List.of(row("raw", 1, 10, null, 1, LATER)));

    AssetDownloadCountData stored = read("raw", 1);
    assertThat(stored.getLastDownloaded().isEqual(LATER), is(true));
    assertThat(stored.getComponentId(), is(nullValue()));
  }

  @Test
  public void keepsTheCountsOfEachFormatApart() {
    underTest.addDownloads(List.of(row("raw", 1, 10, null, 3, EARLIER), row("maven2", 1, 10, null, 5, EARLIER)));

    assertThat(read("raw", 1).getDownloadCount(), is(3L));
    assertThat(read("maven2", 1).getDownloadCount(), is(5L));
  }

  @Test
  public void browsesOnlyTheRequestedAssets() {
    underTest.addDownloads(List.of(
        row("raw", 1, 10, null, 1, EARLIER),
        row("raw", 2, 10, null, 2, EARLIER),
        row("raw", 3, 10, null, 3, EARLIER)));

    List<AssetDownloadCountData> rows = underTest.browse("raw", List.of(1, 3, 4));

    assertThat(rows.stream().map(AssetDownloadCountData::getAssetId).toList(), containsInAnyOrder(1, 3));
    assertThat(underTest.browse("raw", List.of()), is(empty()));
  }

  @Test
  public void browsesMoreAssetsThanFitIntoOneQuery() {
    int assets = AssetDownloadCountStore.BATCH_SIZE + 500;
    underTest.addDownloads(IntStream.rangeClosed(1, assets)
        .mapToObj(assetId -> row("raw", assetId, 10, null, assetId, EARLIER))
        .toList());

    Map<Integer, Long> counts = underTest.browse("raw", IntStream.rangeClosed(1, assets).boxed().toList())
        .stream()
        .collect(Collectors.toMap(AssetDownloadCountData::getAssetId, AssetDownloadCountData::getDownloadCount));

    assertThat(counts.size(), is(assets));
    assertThat(counts.get(assets), is((long) assets));
  }

  @Test
  public void deletesTheRowsOfAssets() {
    underTest.addDownloads(List.of(
        row("raw", 1, 10, null, 1, EARLIER),
        row("raw", 2, 10, null, 2, EARLIER),
        row("maven2", 1, 10, null, 3, EARLIER)));

    assertThat(underTest.deleteAssets("raw", List.of(1)), is(1));

    assertThat(ids("raw", 1, 2), containsInAnyOrder(2));
    assertThat(ids("maven2", 1), containsInAnyOrder(1));
  }

  @Test
  public void deletesTheRowsOfComponents() {
    underTest.addDownloads(List.of(
        row("raw", 1, 10, 100, 1, EARLIER),
        row("raw", 2, 10, 100, 1, EARLIER),
        row("raw", 3, 10, 200, 1, EARLIER),
        row("raw", 4, 10, null, 1, EARLIER),
        row("maven2", 1, 10, 100, 1, EARLIER)));

    assertThat(underTest.deleteComponents("raw", Set.of(100)), is(2));

    assertThat(ids("raw", 1, 2, 3, 4), containsInAnyOrder(3, 4));
    assertThat(ids("maven2", 1), hasSize(1));
  }

  @Test
  public void deletesTheRowsOfARepository() {
    underTest.addDownloads(List.of(
        row("raw", 1, 10, null, 1, EARLIER),
        row("raw", 2, 11, null, 1, EARLIER),
        row("maven2", 3, 10, null, 1, EARLIER)));

    assertThat(underTest.deleteRepository("raw", 10), is(1));

    assertThat(ids("raw", 1, 2), containsInAnyOrder(2));
    assertThat(ids("maven2", 3), containsInAnyOrder(3));
  }

  @Test
  public void deletingNothingIsHarmless() {
    assertThat(underTest.deleteAssets("raw", List.of()), is(0));
    assertThat(underTest.deleteComponents("raw", Set.of()), is(0));
    assertThat(underTest.deleteRepository("raw", 10), is(0));
  }

  private AssetDownloadCountData read(final String format, final int assetId) {
    List<AssetDownloadCountData> rows = underTest.browse(format, List.of(assetId));
    assertThat(rows, hasSize(1));
    return rows.get(0);
  }

  private List<Integer> ids(final String format, final Integer... assetIds) {
    return underTest.browse(format, List.of(assetIds)).stream()
        .map(AssetDownloadCountData::getAssetId)
        .toList();
  }

  private static AssetDownloadCountData row(
      final String format,
      final int assetId,
      final int repositoryId,
      final Integer componentId,
      final long downloads,
      final OffsetDateTime lastDownloaded)
  {
    return new AssetDownloadCountData(format, assetId, repositoryId, componentId, downloads, lastDownloaded);
  }
}
