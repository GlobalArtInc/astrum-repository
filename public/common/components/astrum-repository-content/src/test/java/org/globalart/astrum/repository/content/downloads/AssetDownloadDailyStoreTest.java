package org.globalart.astrum.repository.content.downloads;

import java.time.LocalDate;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.stream.Collectors;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;
import static org.globalart.astrum.repository.content.downloads.AssetDownloadDailyData.NO_COMPONENT;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;

public class AssetDownloadDailyStoreTest
{
  private static final LocalDate DAY = LocalDate.of(2026, 10, 1);

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME).access(AssetDownloadDailyDAO.class);

  private AssetDownloadDailyStore underTest;

  @Before
  public void setUp() {
    underTest = new AssetDownloadDailyStore((DataSessionSupplier) sessionRule);
    underTest.deleteBefore(LocalDate.of(3000, 1, 1));
  }

  @Test(expected = Test.None.class)
  public void canCreateTheSchemaMoreThanOnce() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      session.access(AssetDownloadDailyDAO.class).createSchema();
      session.access(AssetDownloadDailyDAO.class).createSchema();
    }
  }

  @Test
  public void addsTheDownloadsOfADayToTheStoredCount() {
    underTest.addDownloads(List.of(row(DAY, "raw", 10, 100, 3)));
    underTest.addDownloads(List.of(row(DAY, "raw", 10, 100, 2), row(DAY, "raw", 10, 100, 4)));

    assertThat(perDay(DAY), is(Map.of("raw:10:" + DAY, 9L)));
  }

  @Test
  public void keepsDaysFormatsRepositoriesAndComponentsApart() {
    underTest.addDownloads(List.of(
        row(DAY, "raw", 10, 100, 1),
        row(DAY, "raw", 10, 200, 2),
        row(DAY, "raw", 10, NO_COMPONENT, 4),
        row(DAY, "raw", 11, 100, 8),
        row(DAY, "maven2", 10, 100, 16),
        row(DAY.plusDays(1), "raw", 10, 100, 32)));

    assertThat(perDay(DAY), is(Map.of(
        "raw:10:" + DAY, 7L,
        "raw:11:" + DAY, 8L,
        "maven2:10:" + DAY, 16L,
        "raw:10:" + DAY.plusDays(1), 32L)));
    assertThat(perDay(DAY.plusDays(1)), is(Map.of("raw:10:" + DAY.plusDays(1), 32L)));
  }

  @Test
  public void sumsUpTheDownloadsAndComponentsOfEachRepositorySinceADay() {
    underTest.addDownloads(List.of(
        row(DAY.minusDays(1), "raw", 10, 300, 100),
        row(DAY, "raw", 10, 100, 1),
        row(DAY, "raw", 10, NO_COMPONENT, 2),
        row(DAY.plusDays(1), "raw", 10, 100, 4),
        row(DAY.plusDays(1), "raw", 10, 200, 8),
        row(DAY, "raw", 11, NO_COMPONENT, 16)));

    Map<String, RepositoryDownloads> totals = underTest.browseRepositoryTotals(DAY).stream()
        .collect(Collectors.toMap(row -> row.getFormat() + ':' + row.getRepositoryId(), row -> row));

    assertThat(totals.keySet(), is(Set.of("raw:10", "raw:11")));
    assertThat(totals.get("raw:10").getDownloadCount(), is(15L));
    assertThat(totals.get("raw:10").getComponentCount(), is(2L));
    assertThat(totals.get("raw:11").getDownloadCount(), is(16L));
    assertThat(totals.get("raw:11").getComponentCount(), is(0L));
  }

  @Test
  public void listsTheMostDownloadedComponentsOfTheGivenRepositories() {
    underTest.addDownloads(List.of(
        row(DAY.minusDays(1), "raw", 10, 400, 1000),
        row(DAY, "raw", 10, 100, 5),
        row(DAY.plusDays(1), "raw", 10, 100, 5),
        row(DAY, "raw", 10, 200, 20),
        row(DAY, "raw", 10, 300, 1),
        row(DAY, "raw", 10, NO_COMPONENT, 500),
        row(DAY, "raw", 12, 500, 900),
        row(DAY, "raw", 11, 600, 3),
        row(DAY, "maven2", 10, 700, 800)));

    List<ComponentDownloads> top = underTest.browseTopComponents(DAY, "raw", List.of(10, 11), 3);

    assertThat(top.stream().map(row -> row.getComponentId() + "=" + row.getDownloadCount()).toList(),
        contains("200=20", "100=10", "600=3"));
    assertThat(top.get(2).getRepositoryId(), is(11));
    assertThat(underTest.browseTopComponents(DAY, "raw", List.of(), 3), is(empty()));
  }

  @Test
  public void deletesTheDaysBeforeADay() {
    underTest.addDownloads(List.of(
        row(DAY.minusDays(2), "raw", 10, 100, 1),
        row(DAY.minusDays(1), "raw", 10, 100, 2),
        row(DAY, "raw", 10, 100, 4)));

    assertThat(underTest.deleteBefore(DAY.minusDays(1)), is(1));

    assertThat(perDay(DAY.minusDays(5)), is(Map.of(
        "raw:10:" + DAY.minusDays(1), 2L,
        "raw:10:" + DAY, 4L)));
  }

  @Test
  public void deletesTheDaysOfARepository() {
    underTest.addDownloads(List.of(
        row(DAY, "raw", 10, 100, 1),
        row(DAY.plusDays(1), "raw", 10, NO_COMPONENT, 2),
        row(DAY, "raw", 11, 100, 4),
        row(DAY, "maven2", 10, 100, 8)));

    assertThat(underTest.deleteRepository("raw", 10), is(2));

    assertThat(perDay(DAY), is(Map.of(
        "raw:11:" + DAY, 4L,
        "maven2:10:" + DAY, 8L)));
  }

  private Map<String, Long> perDay(final LocalDate from) {
    return underTest.browseRepositoryDays(from).stream().collect(Collectors.toMap(
        row -> row.getFormat() + ':' + row.getRepositoryId() + ':' + row.getDownloadDay(),
        RepositoryDownloads::getDownloadCount));
  }

  private static AssetDownloadDailyData row(
      final LocalDate day,
      final String format,
      final int repositoryId,
      final int componentId,
      final long downloads)
  {
    return new AssetDownloadDailyData(day, format, repositoryId, componentId, downloads);
  }
}
