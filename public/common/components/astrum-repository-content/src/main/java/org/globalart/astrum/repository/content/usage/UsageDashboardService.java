package org.globalart.astrum.repository.content.usage;

import java.time.Clock;
import java.time.LocalDate;
import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.Set;
import java.util.TreeMap;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.function.Predicate;
import java.util.function.Supplier;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.downloads.AssetDownloadCountStore;
import org.globalart.astrum.repository.content.downloads.AssetDownloadCounter;
import org.globalart.astrum.repository.content.downloads.AssetDownloadDailyStore;
import org.globalart.astrum.repository.content.downloads.ComponentDownloads;
import org.globalart.astrum.repository.content.downloads.RepositoryDownloads;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.DayXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.FormatXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.RepositoryUsageXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.RepositoryXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.TotalsXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.UnusedRepositoryXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.UnusedXO;
import org.globalart.astrum.repository.content.usage.UsageDashboardXO.UsageComponentXO;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.RepositoryViewPermission;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.security.SecurityHelper;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.base.Suppliers;
import com.google.common.base.Throwables;
import com.google.common.cache.Cache;
import com.google.common.cache.CacheBuilder;
import com.google.common.util.concurrent.UncheckedExecutionException;
import org.apache.shiro.authz.Permission;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.base.Strings.emptyToNull;
import static org.apache.commons.lang3.StringUtils.strip;
import static org.globalart.astrum.repository.content.store.InternalIds.contentRepositoryId;
import static org.globalart.astrum.repository.content.store.InternalIds.toExternalId;
import static org.globalart.astrum.security.BreadActions.BROWSE;

/**
 * Sums up how much of what is used in the repositories the current user may browse, for the welcome page.
 *
 * A user sees the repositories whose repository-view browse permission they hold, the permission the browse tree asks
 * for; administrators hold it for every repository. Repositories a user reaches only through content selectors are
 * left out, since their totals would include the content the selectors hide.
 *
 * Counting components, assets and their sizes reads every asset, so these figures are read for all repositories at
 * once and kept for {@code astrum.usage.cacheSeconds} seconds. The figures of one set of visible repositories, one
 * period and one repository filter are kept for as long. Each per-format query is bounded: the top lists read the
 * most downloaded components of the visible repositories with a limit, the latest components come from the end of the
 * primary key, and the names of all listed components of a format are read with one query.
 */
@Component
public class UsageDashboardService
{
  static final int TOP = 10;

  static final int UNUSED_TOP = 5;

  /**
   * Extra rows read for the top lists, standing in for components deleted since they were downloaded.
   */
  private static final int SPARE = 10;

  private static final String CACHE_SECONDS_KEY = "astrum.usage.cacheSeconds";

  private static final String UNUSED_DAYS_KEY = "astrum.usage.unusedDays";

  private final Logger log = LoggerFactory.getLogger(getClass());

  private final RepositoryManager repositoryManager;

  private final SecurityHelper securityHelper;

  private final AssetDownloadCounter downloadCounter;

  private final AssetDownloadCountStore downloadCountStore;

  private final AssetDownloadDailyStore downloadDailyStore;

  private final UsageStore usageStore;

  private final int unusedDays;

  private final Clock clock;

  private final Cache<Scope, UsageDashboardXO> dashboards;

  private final Supplier<Map<RepositoryKey, RepositoryContentUsage>> contentUsage;

  @Autowired
  public UsageDashboardService(
      final RepositoryManager repositoryManager,
      final SecurityHelper securityHelper,
      final AssetDownloadCounter downloadCounter,
      final AssetDownloadCountStore downloadCountStore,
      final AssetDownloadDailyStore downloadDailyStore,
      final UsageStore usageStore,
      @Value("${" + CACHE_SECONDS_KEY + ":60}") final int cacheSeconds,
      @Value("${" + UNUSED_DAYS_KEY + ":90}") final int unusedDays)
  {
    this(repositoryManager, securityHelper, downloadCounter, downloadCountStore, downloadDailyStore, usageStore,
        cacheSeconds, unusedDays, Clock.systemUTC());
  }

  @VisibleForTesting
  UsageDashboardService(
      final RepositoryManager repositoryManager,
      final SecurityHelper securityHelper,
      final AssetDownloadCounter downloadCounter,
      final AssetDownloadCountStore downloadCountStore,
      final AssetDownloadDailyStore downloadDailyStore,
      final UsageStore usageStore,
      final int cacheSeconds,
      final int unusedDays,
      final Clock clock)
  {
    this.repositoryManager = checkNotNull(repositoryManager);
    this.securityHelper = checkNotNull(securityHelper);
    this.downloadCounter = checkNotNull(downloadCounter);
    this.downloadCountStore = checkNotNull(downloadCountStore);
    this.downloadDailyStore = checkNotNull(downloadDailyStore);
    this.usageStore = checkNotNull(usageStore);
    checkArgument(cacheSeconds >= 0, CACHE_SECONDS_KEY + " must not be negative");
    checkArgument(unusedDays > 0, UNUSED_DAYS_KEY + " must be positive");
    this.unusedDays = unusedDays;
    this.clock = checkNotNull(clock);
    this.dashboards = CacheBuilder.newBuilder()
        .expireAfterWrite(cacheSeconds, TimeUnit.SECONDS)
        .maximumSize(cacheSeconds > 0 ? 200 : 0)
        .build();
    this.contentUsage = cacheSeconds > 0
        ? Suppliers.memoizeWithExpiration(this::readContentUsage, cacheSeconds, TimeUnit.SECONDS)
        : this::readContentUsage;
  }

  /**
   * Sums up the usage of the repositories the current user may browse over the last {@code days} days, today
   * included, limited to the named repository when one is given.
   *
   * @param days       length of the period, between 1 and the days of downloads per day that are kept
   * @param repository the repository to limit the figures to, or {@code null} for all repositories
   * @return the figures, or nothing when the user may not browse the named repository or it does not exist
   */
  public Optional<UsageDashboardXO> dashboard(final int days, @Nullable final String repository) {
    int period = Math.max(1, Math.min(days, downloadCounter.getRetentionDays()));
    List<Repository> visible = visibleRepositories();
    List<Repository> selected = repository == null ? visible
        : visible.stream().filter(each -> each.getName().equals(repository)).toList();
    if (selected.isEmpty() && repository != null) {
      return Optional.empty();
    }
    Scope scope = new Scope(period, repository, visible.stream().map(Repository::getName).sorted().toList());
    try {
      return Optional.of(dashboards.get(scope, () -> compute(period, repository, visible, selected)));
    }
    catch (ExecutionException | UncheckedExecutionException e) {
      Throwables.throwIfUnchecked(e.getCause());
      throw new IllegalStateException(e.getCause());
    }
  }

  private List<Repository> visibleRepositories() {
    List<Repository> repositories = new ArrayList<>();
    repositoryManager.browse().forEach(repositories::add);
    if (repositories.isEmpty()) {
      return repositories;
    }
    Permission[] permissions = repositories.stream()
        .map(repository -> new RepositoryViewPermission(repository, BROWSE))
        .toArray(Permission[]::new);
    boolean[] permitted = securityHelper.isPermitted(securityHelper.subject(), permissions);
    List<Repository> visible = new ArrayList<>();
    for (int i = 0; i < permitted.length; i++) {
      if (permitted[i]) {
        visible.add(repositories.get(i));
      }
    }
    visible.sort(Comparator.comparing(Repository::getName));
    return visible;
  }

  private UsageDashboardXO compute(
      final int days,
      @Nullable final String repositoryFilter,
      final List<Repository> visible,
      final List<Repository> selected)
  {
    writePendingDownloads();
    LocalDate today = LocalDate.now(clock);
    LocalDate from = today.minusDays(days - 1L);

    Map<RepositoryKey, Repository> byKey = new LinkedHashMap<>();
    selected.forEach(repository -> keyOf(repository).ifPresent(key -> byKey.put(key, repository)));
    Map<String, Set<Integer>> idsByFormat = new TreeMap<>();
    byKey.keySet().forEach(key -> idsByFormat.computeIfAbsent(key.format(), format -> new LinkedHashSet<>())
        .add(key.repositoryId()));

    Map<RepositoryKey, RepositoryContentUsage> content = contentUsage.get();
    Downloads downloads = readDownloads(byKey.keySet(), today, from);

    List<UsageComponentXO> topComponents = new ArrayList<>();
    List<UsageComponentXO> topComponentsAllTime = new ArrayList<>();
    List<UsageComponentXO> recentComponents = new ArrayList<>();
    readComponents(byKey, idsByFormat, from, topComponents, topComponentsAllTime, recentComponents);

    return new UsageDashboardXO(
        days,
        from.toString(),
        today.toString(),
        downloadCounter.getRetentionDays(),
        unusedDays,
        repositoryFilter,
        visible.stream().map(each -> new RepositoryXO(each.getName(), formatOf(each), typeOf(each))).toList(),
        totals(selected, byKey, content, downloads),
        downloads.perDay(from, today),
        topComponents,
        topComponentsAllTime,
        topRepositories(byKey, content, downloads,
            Comparator.comparingLong(RepositoryUsageXO::downloads), usage -> usage.downloads() > 0),
        topRepositories(byKey, content, downloads,
            Comparator.comparingLong(RepositoryUsageXO::storageBytes), usage -> usage.storageBytes() > 0),
        recentComponents,
        unused(byKey, content));
  }

  private void writePendingDownloads() {
    try {
      downloadCounter.flush();
    }
    catch (RuntimeException e) {
      log.warn("Failed to write the pending downloads before summing them up; they are left out for now", e);
    }
  }

  private TotalsXO totals(
      final List<Repository> selected,
      final Map<RepositoryKey, Repository> byKey,
      final Map<RepositoryKey, RepositoryContentUsage> content,
      final Downloads downloads)
  {
    Map<String, int[]> typesByFormat = new TreeMap<>();
    int[] types = new int[3];
    for (Repository repository : selected) {
      int type = typeIndex(repository);
      if (type >= 0) {
        types[type]++;
        typesByFormat.computeIfAbsent(formatOf(repository), format -> new int[3])[type]++;
      }
    }
    long components = 0;
    long assets = 0;
    long bytes = 0;
    long proxyBytes = 0;
    for (Map.Entry<RepositoryKey, Repository> entry : byKey.entrySet()) {
      RepositoryContentUsage usage = content.get(entry.getKey());
      if (usage != null) {
        components += usage.getComponents();
        assets += usage.getAssets();
        bytes += usage.getBytes();
        if (ProxyType.NAME.equals(typeOf(entry.getValue()))) {
          proxyBytes += usage.getBytes();
        }
      }
    }
    return new TotalsXO(
        selected.size(),
        types[0],
        types[1],
        types[2],
        typesByFormat.entrySet().stream()
            .map(each -> new FormatXO(each.getKey(), each.getValue()[0], each.getValue()[1], each.getValue()[2]))
            .toList(),
        components,
        assets,
        bytes,
        proxyBytes,
        downloads.allTime(),
        downloads.today(),
        downloads.last7Days(),
        downloads.last30Days(),
        downloads.inPeriod(),
        downloads.componentsInPeriod());
  }

  private List<RepositoryUsageXO> topRepositories(
      final Map<RepositoryKey, Repository> byKey,
      final Map<RepositoryKey, RepositoryContentUsage> content,
      final Downloads downloads,
      final Comparator<RepositoryUsageXO> order,
      final Predicate<RepositoryUsageXO> listed)
  {
    return byKey.entrySet().stream()
        .map(entry -> {
          RepositoryContentUsage usage = content.get(entry.getKey());
          Repository repository = entry.getValue();
          return new RepositoryUsageXO(repository.getName(), formatOf(repository), typeOf(repository),
              downloads.inPeriodOf(entry.getKey()),
              usage != null ? usage.getBytes() : 0, usage != null ? usage.getComponents() : 0);
        })
        .filter(listed)
        .sorted(order.reversed().thenComparing(RepositoryUsageXO::name))
        .limit(TOP)
        .toList();
  }

  private UnusedXO unused(
      final Map<RepositoryKey, Repository> byKey,
      final Map<RepositoryKey, RepositoryContentUsage> content)
  {
    long components = 0;
    long bytes = 0;
    long neverComponents = 0;
    long neverBytes = 0;
    List<UnusedRepositoryXO> repositories = new ArrayList<>();
    for (Map.Entry<RepositoryKey, Repository> entry : byKey.entrySet()) {
      RepositoryContentUsage usage = content.get(entry.getKey());
      if (usage == null) {
        continue;
      }
      components += usage.getUnusedComponents();
      bytes += usage.getUnusedBytes();
      neverComponents += usage.getNeverDownloadedComponents();
      neverBytes += usage.getNeverDownloadedBytes();
      if (usage.getUnusedComponents() > 0) {
        repositories.add(new UnusedRepositoryXO(entry.getValue().getName(), formatOf(entry.getValue()),
            usage.getUnusedComponents(), usage.getUnusedBytes()));
      }
    }
    repositories.sort(Comparator.comparingLong(UnusedRepositoryXO::bytes)
        .thenComparingLong(UnusedRepositoryXO::components)
        .reversed()
        .thenComparing(UnusedRepositoryXO::name));
    return new UnusedXO(components, bytes, neverComponents, neverBytes,
        repositories.stream().limit(UNUSED_TOP).toList());
  }

  private Downloads readDownloads(final Set<RepositoryKey> keys, final LocalDate today, final LocalDate from) {
    LocalDate last30From = today.minusDays(29);
    LocalDate readFrom = from.isBefore(last30From) ? from : last30From;
    Downloads downloads = new Downloads(today, from);
    for (RepositoryDownloads row : downloadDailyStore.browseRepositoryDays(readFrom)) {
      RepositoryKey key = new RepositoryKey(row.getFormat(), row.getRepositoryId());
      if (keys.contains(key) && row.getDownloadDay() != null) {
        downloads.addDay(key, row.getDownloadDay(), row.getDownloadCount());
      }
    }
    for (RepositoryDownloads row : downloadDailyStore.browseRepositoryTotals(from)) {
      if (keys.contains(new RepositoryKey(row.getFormat(), row.getRepositoryId()))) {
        downloads.componentsInPeriod += row.getComponentCount();
      }
    }
    for (RepositoryDownloads row : downloadCountStore.browseRepositoryTotals()) {
      if (keys.contains(new RepositoryKey(row.getFormat(), row.getRepositoryId()))) {
        downloads.allTime += row.getDownloadCount();
      }
    }
    return downloads;
  }

  private void readComponents(
      final Map<RepositoryKey, Repository> byKey,
      final Map<String, Set<Integer>> idsByFormat,
      final LocalDate from,
      final List<UsageComponentXO> topComponents,
      final List<UsageComponentXO> topComponentsAllTime,
      final List<UsageComponentXO> recentComponents)
  {
    Map<String, Set<Integer>> allIdsByFormat = allContentRepositoryIds();
    List<ComponentDownloads> inPeriod = new ArrayList<>();
    List<ComponentDownloads> allTime = new ArrayList<>();
    List<ComponentKey> latest = new ArrayList<>();
    idsByFormat.forEach((format, ids) -> {
      inPeriod.addAll(downloadDailyStore.browseTopComponents(from, format, ids, TOP + SPARE));
      allTime.addAll(downloadCountStore.browseTopComponents(format, ids, TOP + SPARE));
      Collection<Integer> filter = ids.equals(allIdsByFormat.get(format)) ? null : ids;
      usageStore.browseLatestComponentIds(format, filter, TOP)
          .forEach(componentId -> latest.add(new ComponentKey(format, componentId)));
    });

    Map<String, Set<Integer>> wanted = new TreeMap<>();
    inPeriod.forEach(row -> wanted.computeIfAbsent(row.getFormat(), f -> new LinkedHashSet<>()).add(row.getComponentId()));
    allTime.forEach(row -> wanted.computeIfAbsent(row.getFormat(), f -> new LinkedHashSet<>()).add(row.getComponentId()));
    latest.forEach(key -> wanted.computeIfAbsent(key.format(), f -> new LinkedHashSet<>()).add(key.componentId()));
    Map<ComponentKey, ComponentSummary> summaries = new HashMap<>();
    wanted.forEach((format, ids) -> usageStore.browseComponents(format, ids)
        .forEach(summary -> summaries.put(new ComponentKey(format, summary.getComponentId()), summary)));

    topComponents.addAll(topList(inPeriod, summaries, byKey));
    topComponentsAllTime.addAll(topList(allTime, summaries, byKey));
    latest.stream()
        .filter(summaries::containsKey)
        .sorted(Comparator.comparing((ComponentKey key) -> summaries.get(key).getCreated(),
                Comparator.nullsFirst(Comparator.naturalOrder()))
            .thenComparingInt(ComponentKey::componentId)
            .reversed())
        .map(key -> toXO(key.format(), summaries.get(key), 0, byKey))
        .filter(Objects::nonNull)
        .limit(TOP)
        .forEach(recentComponents::add);
  }

  private List<UsageComponentXO> topList(
      final List<ComponentDownloads> rows,
      final Map<ComponentKey, ComponentSummary> summaries,
      final Map<RepositoryKey, Repository> byKey)
  {
    return rows.stream()
        .sorted(Comparator.comparingLong(ComponentDownloads::getDownloadCount).reversed()
            .thenComparing(ComponentDownloads::getFormat)
            .thenComparingInt(ComponentDownloads::getComponentId))
        .map(row -> toXO(row.getFormat(), summaries.get(new ComponentKey(row.getFormat(), row.getComponentId())),
            row.getDownloadCount(), byKey))
        .filter(Objects::nonNull)
        .limit(TOP)
        .toList();
  }

  @Nullable
  private static UsageComponentXO toXO(
      final String format,
      @Nullable final ComponentSummary summary,
      final long downloads,
      final Map<RepositoryKey, Repository> byKey)
  {
    if (summary == null) {
      return null;
    }
    Repository repository = byKey.get(new RepositoryKey(format, summary.getRepositoryId()));
    if (repository == null) {
      return null;
    }
    return new UsageComponentXO(
        toExternalId(summary.getComponentId()).getValue(),
        repository.getName(),
        format,
        emptyToNull(summary.getNamespace()),
        summary.getName(),
        emptyToNull(summary.getVersion()),
        downloads,
        summary.getCreated() != null ? summary.getCreated().toInstant().toString() : "",
        emptyToNull(summary.getCreatedBy()),
        emptyToNull(strip(summary.getBrowsePath(), "/")));
  }

  private Map<String, Set<Integer>> allContentRepositoryIds() {
    Map<String, Set<Integer>> ids = new HashMap<>();
    repositoryManager.browse().forEach(repository -> keyOf(repository).ifPresent(
        key -> ids.computeIfAbsent(key.format(), format -> new LinkedHashSet<>()).add(key.repositoryId())));
    return ids;
  }

  /**
   * Reads what the content of every repository takes up, one format at a time; a format that cannot be read counts as
   * empty, so that one broken format does not hide the others.
   */
  private Map<RepositoryKey, RepositoryContentUsage> readContentUsage() {
    OffsetDateTime unusedSince = OffsetDateTime.now(clock).minusDays(unusedDays);
    Set<String> formats = new LinkedHashSet<>();
    repositoryManager.browse().forEach(repository -> keyOf(repository).ifPresent(key -> formats.add(key.format())));
    Map<RepositoryKey, RepositoryContentUsage> usage = new HashMap<>();
    for (String format : formats) {
      try {
        usageStore.browseContentUsage(format, unusedSince)
            .forEach(row -> usage.put(new RepositoryKey(format, row.getRepositoryId()), row));
        for (RepositoryContentUsage row : usageStore.countComponents(format)) {
          usage.computeIfAbsent(new RepositoryKey(format, row.getRepositoryId()), key -> row)
              .setComponents(row.getComponents());
        }
      }
      catch (RuntimeException e) {
        log.warn("Failed to read what the {} repositories take up", format, e);
      }
    }
    return usage;
  }

  private Optional<RepositoryKey> keyOf(final Repository repository) {
    try {
      return contentRepositoryId(repository).map(id -> new RepositoryKey(formatOf(repository), id));
    }
    catch (RuntimeException e) {
      log.debug("Repository {} has no content to sum up", repository.getName(), e);
      return Optional.empty();
    }
  }

  private static String formatOf(final Repository repository) {
    return repository.getFormat().getValue();
  }

  private static String typeOf(final Repository repository) {
    return repository.getType().getValue();
  }

  private static int typeIndex(final Repository repository) {
    return switch (typeOf(repository)) {
      case HostedType.NAME -> 0;
      case ProxyType.NAME -> 1;
      case GroupType.NAME -> 2;
      default -> -1;
    };
  }

  /**
   * The repositories a set of figures is about.
   */
  private record Scope(int days, @Nullable String repository, List<String> visible)
  {
  }

  private record RepositoryKey(String format, int repositoryId)
  {
  }

  private record ComponentKey(String format, int componentId)
  {
  }

  /**
   * Downloads of a set of repositories, added up while reading.
   */
  private static final class Downloads
  {
    private final LocalDate today;

    private final LocalDate from;

    private final Map<LocalDate, Long> perDay = new HashMap<>();

    private final Map<RepositoryKey, Long> inPeriodByRepository = new HashMap<>();

    private long allTime;

    private long componentsInPeriod;

    private Downloads(final LocalDate today, final LocalDate from) {
      this.today = today;
      this.from = from;
    }

    private void addDay(final RepositoryKey key, final LocalDate day, final long count) {
      perDay.merge(day, count, Long::sum);
      if (!day.isBefore(from)) {
        inPeriodByRepository.merge(key, count, Long::sum);
      }
    }

    private long since(final LocalDate first) {
      return perDay.entrySet().stream()
          .filter(entry -> !entry.getKey().isBefore(first) && !entry.getKey().isAfter(today))
          .mapToLong(Map.Entry::getValue)
          .sum();
    }

    private long allTime() {
      return allTime;
    }

    private long today() {
      return since(today);
    }

    private long last7Days() {
      return since(today.minusDays(6));
    }

    private long last30Days() {
      return since(today.minusDays(29));
    }

    private long inPeriod() {
      return since(from);
    }

    private long componentsInPeriod() {
      return componentsInPeriod;
    }

    private long inPeriodOf(final RepositoryKey key) {
      return inPeriodByRepository.getOrDefault(key, 0L);
    }

    private List<DayXO> perDay(final LocalDate first, final LocalDate last) {
      List<DayXO> days = new ArrayList<>();
      for (LocalDate day = first; !day.isAfter(last); day = day.plusDays(1)) {
        days.add(new DayXO(day.toString(), perDay.getOrDefault(day, 0L)));
      }
      return days;
    }
  }
}
