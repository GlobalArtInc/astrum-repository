package org.globalart.astrum.repository.content.usage;

import java.time.Clock;
import java.time.Instant;
import java.time.LocalDate;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.ArrayList;
import java.util.Collection;
import java.util.List;
import java.util.Optional;
import java.util.stream.IntStream;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.content.downloads.AssetDownloadCountStore;
import org.globalart.astrum.repository.content.downloads.AssetDownloadCounter;
import org.globalart.astrum.repository.content.downloads.AssetDownloadDailyStore;
import org.globalart.astrum.repository.content.downloads.ComponentDownloads;
import org.globalart.astrum.repository.content.downloads.RepositoryDownloads;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.DayXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.RepositoryUsageXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.RepositoryXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.UnusedRepositoryXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.UsageComponentXO;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.RepositoryViewPermission;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.security.SecurityHelper;

import org.apache.shiro.authz.Permission;
import org.apache.shiro.subject.Subject;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.InOrder;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.globalart.astrum.repository.content.store.InternalIds.toExternalId;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.everyItem;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyCollection;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.ArgumentMatchers.isNull;
import static org.mockito.Mockito.inOrder;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class UsageDashboardServiceTest
{
  private static final Instant NOW = Instant.parse("2026-10-11T12:00:00Z");

  private static final LocalDate TODAY = LocalDate.of(2026, 10, 11);

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private SecurityHelper securityHelper;

  @Mock
  private Subject subject;

  @Mock
  private AssetDownloadCounter downloadCounter;

  @Mock
  private AssetDownloadCountStore downloadCountStore;

  @Mock
  private AssetDownloadDailyStore downloadDailyStore;

  @Mock
  private UsageStore usageStore;

  private final List<Repository> repositories = new ArrayList<>();

  private boolean[] permitted;

  private Repository rawHosted;

  private Repository rawProxy;

  private Repository rawSecret;

  private Repository mavenHosted;

  private Repository mavenGroup;

  @Before
  public void setUp() {
    rawHosted = repository("raw-hosted", "raw", new HostedType(), 1);
    rawProxy = repository("raw-proxy", "raw", new ProxyType(), 2);
    rawSecret = repository("raw-secret", "raw", new HostedType(), 3);
    mavenHosted = repository("maven-hosted", "maven2", new HostedType(), 1);
    mavenGroup = repository("maven-group", "maven2", new GroupType(), null);
    when(repositoryManager.browse()).thenReturn(repositories);
    when(securityHelper.subject()).thenReturn(subject);
    permitAll();
    when(securityHelper.isPermitted(eq(subject), any(Permission[].class))).thenAnswer(invocation -> {
      Permission[] permissions = (Permission[]) invocation.getRawArguments()[1];
      boolean[] result = new boolean[permissions.length];
      for (int i = 0; i < permissions.length; i++) {
        RepositoryViewPermission permission = (RepositoryViewPermission) permissions[i];
        assertThat(permission.getActions(), contains("browse"));
        result[i] = permitted[indexOf(permission.getName())];
      }
      return result;
    });
    when(downloadCounter.getRetentionDays()).thenReturn(180);

    when(usageStore.browseContentUsage(eq("raw"), any())).thenReturn(List.of(
        content(1, 40, 100_000, 3, 3000, 1, 500),
        content(2, 20, 900_000, 2, 2000, 0, 0),
        content(3, 7, 50_000_000, 7, 50_000_000, 7, 50_000_000)));
    when(usageStore.countComponents("raw")).thenReturn(List.of(components(1, 10), components(2, 5),
        components(3, 7)));
    when(usageStore.browseContentUsage(eq("maven2"), any())).thenReturn(List.of(
        content(1, 30, 300_000, 4, 4000, 2, 100)));
    when(usageStore.countComponents("maven2")).thenReturn(List.of(components(1, 6)));

    when(downloadDailyStore.browseRepositoryDays(any())).thenReturn(List.of(
        day("raw", 1, TODAY, 5),
        day("raw", 1, TODAY.minusDays(2), 7),
        day("raw", 2, TODAY.minusDays(2), 1),
        day("raw", 3, TODAY, 1000),
        day("maven2", 1, TODAY.minusDays(6), 11),
        day("maven2", 1, TODAY.minusDays(20), 13),
        day("raw", 1, TODAY.minusDays(29), 17)));
    when(downloadDailyStore.browseRepositoryTotals(any())).thenReturn(List.of(
        totals("raw", 1, 12, 3),
        totals("raw", 3, 1000, 9),
        totals("maven2", 1, 11, 2)));
    when(downloadCountStore.browseRepositoryTotals()).thenReturn(List.of(
        totals("raw", 1, 100, 0),
        totals("raw", 2, 50, 0),
        totals("raw", 3, 5000, 0),
        totals("maven2", 1, 70, 0)));
    when(usageStore.browseComponents(anyString(), anyCollection())).thenAnswer(invocation -> {
      String format = invocation.getArgument(0);
      Collection<Integer> ids = invocation.getArgument(1);
      return ids.stream()
          .filter(id -> id != 404)
          .map(id -> summary(format, id))
          .toList();
    });
  }

  @Test
  public void showsOnlyTheRepositoriesTheUserMayBrowse() {
    permitted = new boolean[]{true, true, false, true, true};
    when(downloadDailyStore.browseTopComponents(any(), eq("raw"), anyCollection(), anyInt()))
        .thenReturn(List.of(downloads("raw", 1, 11, 9)));

    UsageDashboardXO dashboard = underTest(0).dashboard(30, null).orElseThrow();

    assertThat(dashboard.repositories().stream().map(RepositoryXO::name).toList(),
        contains("maven-group", "maven-hosted", "raw-hosted", "raw-proxy"));
    assertThat(dashboard.totals().repositories(), is(4));
    assertThat(dashboard.totals().hosted(), is(2));
    assertThat(dashboard.totals().proxy(), is(1));
    assertThat(dashboard.totals().group(), is(1));
    assertThat(dashboard.totals().components(), is(21L));
    assertThat(dashboard.totals().assets(), is(90L));
    assertThat(dashboard.totals().storageBytes(), is(1_300_000L));
    assertThat(dashboard.totals().proxyStorageBytes(), is(900_000L));
    assertThat(dashboard.totals().downloads(), is(220L));
    assertThat(dashboard.totals().componentsDownloadedInPeriod(), is(5L));
    assertThat(dashboard.unused().components(), is(9L));
    assertThat(dashboard.unused().repositories().stream().map(UnusedRepositoryXO::name).toList(),
        contains("maven-hosted", "raw-hosted", "raw-proxy"));

    ArgumentCaptor<Collection<Integer>> rawIds = ArgumentCaptor.forClass(Collection.class);
    verify(downloadDailyStore).browseTopComponents(eq(TODAY.minusDays(29)), eq("raw"), rawIds.capture(), eq(20));
    assertThat(rawIds.getValue(), containsInAnyOrder(1, 2));
    verify(usageStore).browseLatestComponentIds(eq("raw"), eq(rawIds.getValue()), eq(10));
    verify(usageStore).browseLatestComponentIds(eq("maven2"), isNull(), eq(10));
  }

  @Test
  public void showsAdministratorsEveryRepository() {
    UsageDashboardXO dashboard = underTest(0).dashboard(30, null).orElseThrow();

    assertThat(dashboard.totals().repositories(), is(5));
    assertThat(dashboard.totals().storageBytes(), is(51_300_000L));
    assertThat(dashboard.totals().downloads(), is(5220L));
    assertThat(dashboard.totals().byFormat().stream().map(f -> f.format() + ":" + f.hosted() + "/" + f.proxy() + "/"
        + f.group()).toList(), contains("maven2:1/0/1", "raw:2/1/0"));
    verify(usageStore).browseLatestComponentIds(eq("raw"), isNull(), eq(10));
    verify(usageStore).browseLatestComponentIds(eq("maven2"), isNull(), eq(10));
  }

  @Test
  public void fillsTheDaysWithoutDownloadsWithZeros() {
    permitted = new boolean[]{true, true, false, true, true};

    UsageDashboardXO dashboard = underTest(0).dashboard(7, null).orElseThrow();

    assertThat(dashboard.days(), is(7));
    assertThat(dashboard.from(), is("2026-10-05"));
    assertThat(dashboard.to(), is("2026-10-11"));
    assertThat(dashboard.downloadsPerDay().stream().map(day -> day.day() + "=" + day.downloads()).toList(), contains(
        "2026-10-05=11", "2026-10-06=0", "2026-10-07=0", "2026-10-08=0", "2026-10-09=8", "2026-10-10=0",
        "2026-10-11=5"));
    assertThat(dashboard.totals().downloadsToday(), is(5L));
    assertThat(dashboard.totals().downloadsLast7Days(), is(24L));
    assertThat(dashboard.totals().downloadsLast30Days(), is(54L));
    assertThat(dashboard.totals().downloadsInPeriod(), is(24L));
    verify(downloadDailyStore).browseRepositoryDays(TODAY.minusDays(29));
    verify(downloadDailyStore).browseRepositoryTotals(TODAY.minusDays(6));
  }

  @Test
  public void readsTheDaysOfLongerPeriods() {
    UsageDashboardXO dashboard = underTest(0).dashboard(90, null).orElseThrow();

    assertThat(dashboard.downloadsPerDay(), hasSize(90));
    assertThat(dashboard.downloadsPerDay().get(0).day(), is("2026-07-14"));
    assertThat(dashboard.downloadsPerDay().stream().mapToLong(DayXO::downloads).sum(), is(1054L));
    verify(downloadDailyStore).browseRepositoryDays(TODAY.minusDays(89));
  }

  @Test
  public void keepsThePeriodWithinTheKeptDays() {
    assertThat(underTest(0).dashboard(0, null).orElseThrow().downloadsPerDay(), hasSize(1));
    assertThat(underTest(0).dashboard(1000, null).orElseThrow().downloadsPerDay(), hasSize(180));
  }

  @Test
  public void listsTheMostDownloadedComponentsThatStillExist() {
    when(downloadDailyStore.browseTopComponents(any(), eq("raw"), anyCollection(), anyInt())).thenReturn(List.of(
        downloads("raw", 1, 11, 50), downloads("raw", 1, 404, 40), downloads("raw", 2, 21, 5)));
    when(downloadDailyStore.browseTopComponents(any(), eq("maven2"), anyCollection(), anyInt())).thenReturn(List.of(
        downloads("maven2", 1, 11, 30)));
    when(downloadCountStore.browseTopComponents(eq("raw"), anyCollection(), anyInt())).thenReturn(
        IntStream.rangeClosed(1, 12).mapToObj(i -> downloads("raw", 1, 100 + i, 1000 - i)).toList());

    UsageDashboardXO dashboard = underTest(0).dashboard(30, null).orElseThrow();

    assertThat(dashboard.topComponents().stream().map(c -> c.format() + ":" + c.name() + "=" + c.downloads()).toList(),
        contains("raw:name-11=50", "maven2:name-11=30", "raw:name-21=5"));
    UsageComponentXO top = dashboard.topComponents().get(0);
    assertThat(top.id(), is(toExternalId(11).getValue()));
    assertThat(top.repository(), is("raw-hosted"));
    assertThat(top.group(), is("org.example"));
    assertThat(top.version(), is("1.11"));
    assertThat(top.browsePath(), is("org/example/name-11/1.11"));
    assertThat(top.createdBy(), is("alice"));
    assertThat(dashboard.topComponents().get(1).repository(), is("maven-hosted"));
    assertThat(dashboard.topComponentsAllTime(), hasSize(10));
    assertThat(dashboard.topComponentsAllTime().get(0).downloads(), is(999L));
  }

  @Test
  public void listsTheComponentsAddedLastAcrossFormats() {
    when(usageStore.browseLatestComponentIds(eq("raw"), any(), anyInt())).thenReturn(List.of(31, 30));
    when(usageStore.browseLatestComponentIds(eq("maven2"), any(), anyInt())).thenReturn(List.of(5));

    UsageDashboardXO dashboard = underTest(0).dashboard(30, null).orElseThrow();

    assertThat(dashboard.recentComponents().stream().map(c -> c.format() + ":" + c.name()).toList(),
        contains("raw:name-31", "raw:name-30", "maven2:name-5"));
    assertThat(dashboard.recentComponents().get(0).created(), is("2026-10-11T07:00:00Z"));
    assertThat(dashboard.recentComponents().get(0).downloads(), is(0L));
  }

  @Test
  public void ranksTheRepositoriesByDownloadsAndStorage() {
    permitted = new boolean[]{true, true, false, true, true};

    UsageDashboardXO dashboard = underTest(0).dashboard(30, null).orElseThrow();

    assertThat(dashboard.topRepositoriesByDownloads().stream().map(r -> r.name() + "=" + r.downloads()).toList(),
        contains("raw-hosted=29", "maven-hosted=24", "raw-proxy=1"));
    assertThat(dashboard.topRepositoriesByStorage().stream().map(RepositoryUsageXO::name).toList(),
        contains("raw-proxy", "maven-hosted", "raw-hosted"));
    assertThat(dashboard.topRepositoriesByStorage().get(0).storageBytes(), is(900_000L));
    assertThat(dashboard.topRepositoriesByStorage().get(0).components(), is(5L));
  }

  @Test
  public void limitsTheFiguresToTheNamedRepository() {
    permitted = new boolean[]{true, true, false, true, true};

    UsageDashboardXO dashboard = underTest(0).dashboard(30, "raw-hosted").orElseThrow();

    assertThat(dashboard.repository(), is("raw-hosted"));
    assertThat(dashboard.repositories(), hasSize(4));
    assertThat(dashboard.totals().repositories(), is(1));
    assertThat(dashboard.totals().components(), is(10L));
    assertThat(dashboard.totals().downloads(), is(100L));
    assertThat(dashboard.downloadsPerDay().stream().mapToLong(DayXO::downloads).sum(), is(29L));
    verify(usageStore, never()).browseLatestComponentIds(eq("maven2"), any(), anyInt());
  }

  @Test
  public void hidesRepositoriesTheUserMayNotBrowse() {
    permitted = new boolean[]{true, true, false, true, true};

    assertThat(underTest(0).dashboard(30, "raw-secret"), is(Optional.empty()));
    assertThat(underTest(0).dashboard(30, "missing"), is(Optional.empty()));
  }

  @Test
  public void showsNothingWhenNoRepositoryMayBeBrowsed() {
    permitted = new boolean[]{false, false, false, false, false};

    UsageDashboardXO dashboard = underTest(0).dashboard(30, null).orElseThrow();

    assertThat(dashboard.repositories(), is(empty()));
    assertThat(dashboard.totals().repositories(), is(0));
    assertThat(dashboard.totals().downloads(), is(0L));
    assertThat(dashboard.downloadsPerDay().stream().map(DayXO::downloads).toList(), everyItem(is(0L)));
    assertThat(dashboard.topComponents(), is(empty()));
    assertThat(dashboard.recentComponents(), is(empty()));
    assertThat(dashboard.unused().repositories(), is(empty()));
    assertThat(dashboard.repository(), is(nullValue()));
    verify(usageStore, never()).browseLatestComponentIds(anyString(), any(), anyInt());
  }

  @Test
  public void writesThePendingDownloadsBeforeReadingThem() {
    underTest(0).dashboard(30, null);

    InOrder inOrder = inOrder(downloadCounter, downloadDailyStore);
    inOrder.verify(downloadCounter).flush();
    inOrder.verify(downloadDailyStore).browseRepositoryDays(any());
  }

  @Test
  public void keepsTheFiguresOfEachSetOfRepositoriesForAWhile() {
    UsageDashboardService underTest = underTest(60);

    underTest.dashboard(30, null);
    underTest.dashboard(30, null);
    verify(downloadDailyStore, times(1)).browseRepositoryDays(any());

    permitted = new boolean[]{true, true, false, true, true};
    underTest.dashboard(30, null);
    underTest.dashboard(7, null);
    verify(downloadDailyStore, times(3)).browseRepositoryDays(any());
    verify(usageStore, times(1)).browseContentUsage(eq("raw"), any());
  }

  @Test
  public void judgesUnusedComponentsByTheConfiguredDays() {
    underTest(0).dashboard(30, null);

    verify(usageStore).browseContentUsage("raw", OffsetDateTime.ofInstant(NOW, ZoneOffset.UTC).minusDays(90));
  }

  private UsageDashboardService underTest(final int cacheSeconds) {
    return new UsageDashboardService(repositoryManager, securityHelper, downloadCounter, downloadCountStore,
        downloadDailyStore, usageStore, cacheSeconds, 90, Clock.fixed(NOW, ZoneOffset.UTC));
  }

  private void permitAll() {
    permitted = new boolean[]{true, true, true, true, true};
  }

  private int indexOf(final String name) {
    for (int i = 0; i < repositories.size(); i++) {
      if (repositories.get(i).getName().equals(name)) {
        return i;
      }
    }
    throw new IllegalArgumentException(name);
  }

  private Repository repository(
      final String name,
      final String format,
      final Type type,
      final Integer contentRepositoryId)
  {
    Repository repository = mock(Repository.class);
    when(repository.getName()).thenReturn(name);
    when(repository.getFormat()).thenReturn(new Format(format) { });
    when(repository.getType()).thenReturn(type);
    if (contentRepositoryId != null) {
      ContentFacet contentFacet = mock(ContentFacet.class);
      when(contentFacet.contentRepositoryId()).thenReturn(contentRepositoryId);
      when(repository.optionalFacet(ContentFacet.class)).thenReturn(Optional.of(contentFacet));
    }
    else {
      when(repository.optionalFacet(ContentFacet.class)).thenReturn(Optional.empty());
    }
    repositories.add(repository);
    return repository;
  }

  private static RepositoryContentUsage content(
      final int repositoryId,
      final long assets,
      final long bytes,
      final long unusedComponents,
      final long unusedBytes,
      final long neverDownloadedComponents,
      final long neverDownloadedBytes)
  {
    RepositoryContentUsage usage = new RepositoryContentUsage();
    usage.setRepositoryId(repositoryId);
    usage.setAssets(assets);
    usage.setBytes(bytes);
    usage.setUnusedComponents(unusedComponents);
    usage.setUnusedBytes(unusedBytes);
    usage.setNeverDownloadedComponents(neverDownloadedComponents);
    usage.setNeverDownloadedBytes(neverDownloadedBytes);
    return usage;
  }

  private static RepositoryContentUsage components(final int repositoryId, final long components) {
    RepositoryContentUsage usage = new RepositoryContentUsage();
    usage.setRepositoryId(repositoryId);
    usage.setComponents(components);
    return usage;
  }

  private static RepositoryDownloads day(
      final String format,
      final int repositoryId,
      final LocalDate day,
      final long downloads)
  {
    RepositoryDownloads row = new RepositoryDownloads();
    row.setFormat(format);
    row.setRepositoryId(repositoryId);
    row.setDownloadDay(day);
    row.setDownloadCount(downloads);
    return row;
  }

  private static RepositoryDownloads totals(
      final String format,
      final int repositoryId,
      final long downloads,
      final long components)
  {
    RepositoryDownloads row = new RepositoryDownloads();
    row.setFormat(format);
    row.setRepositoryId(repositoryId);
    row.setDownloadCount(downloads);
    row.setComponentCount(components);
    return row;
  }

  private static ComponentDownloads downloads(
      final String format,
      final int repositoryId,
      final int componentId,
      final long downloads)
  {
    return new ComponentDownloads(format, repositoryId, componentId, downloads);
  }

  private static ComponentSummary summary(final String format, final int componentId) {
    ComponentSummary summary = new ComponentSummary();
    summary.setComponentId(componentId);
    summary.setRepositoryId(componentId >= 20 && componentId < 30 ? 2 : 1);
    summary.setNamespace("org.example");
    summary.setName("name-" + componentId);
    summary.setVersion("1." + componentId);
    summary.setCreated(OffsetDateTime.ofInstant(NOW, ZoneOffset.UTC).minusHours(5).minusMinutes(
        "maven2".equals(format) ? 30 : 0).minusSeconds(componentId >= 30 ? 31 - componentId : 0));
    summary.setCreatedBy("alice");
    summary.setBrowsePath("/org/example/name-" + componentId + "/1." + componentId + "/");
    return summary;
  }
}
