package org.globalart.astrum.repository.content.usage;

import java.util.List;
import javax.annotation.Nullable;

/**
 * How much of what is used in the repositories a user may browse, as shown on the welcome page.
 *
 * Days are UTC days given as {@code yyyy-MM-dd}; the period runs from {@code from} to {@code to}, today, both included.
 *
 * @param days                       length of the period in days
 * @param from                       first day of the period
 * @param to                         last day of the period, today
 * @param retentionDays              how many days of downloads per day are kept
 * @param unusedDays                 how many days without downloads make a component unused
 * @param repository                 the repository the figures are limited to, or {@code null} for all of them
 * @param repositories               the repositories the user may browse, to pick one from
 * @param totals                     totals over the repositories
 * @param downloadsPerDay            downloads on each day of the period, oldest first, days without downloads included
 * @param topComponents              the components downloaded most in the period
 * @param topComponentsAllTime       the components downloaded most since downloads are counted
 * @param topRepositoriesByDownloads the repositories downloaded from most in the period
 * @param topRepositoriesByStorage   the repositories that take up most storage
 * @param recentComponents           the components added last
 * @param unused                     the components not downloaded for {@code unusedDays} days
 */
public record UsageDashboardXO(
    int days,
    String from,
    String to,
    int retentionDays,
    int unusedDays,
    @Nullable String repository,
    List<RepositoryXO> repositories,
    TotalsXO totals,
    List<DayXO> downloadsPerDay,
    List<UsageComponentXO> topComponents,
    List<UsageComponentXO> topComponentsAllTime,
    List<RepositoryUsageXO> topRepositoriesByDownloads,
    List<RepositoryUsageXO> topRepositoriesByStorage,
    List<UsageComponentXO> recentComponents,
    UnusedXO unused)
{
  /**
   * A repository the user may browse.
   */
  public record RepositoryXO(String name, String format, String type)
  {
  }

  /**
   * Totals over the repositories. Storage adds up the sizes of the blobs of the assets; downloads in a period come from
   * the downloads per day, which are kept for {@code retentionDays} days, all-time downloads from the download counts
   * of the assets that are still stored.
   */
  public record TotalsXO(
      int repositories,
      int hosted,
      int proxy,
      int group,
      List<FormatXO> byFormat,
      long components,
      long assets,
      long storageBytes,
      long proxyStorageBytes,
      long downloads,
      long downloadsToday,
      long downloadsLast7Days,
      long downloadsLast30Days,
      long downloadsInPeriod,
      long componentsDownloadedInPeriod)
  {
  }

  /**
   * How many repositories of each type a format has.
   */
  public record FormatXO(String format, int hosted, int proxy, int group)
  {
  }

  /**
   * Downloads on one day.
   */
  public record DayXO(String day, long downloads)
  {
  }

  /**
   * A component, with what the browse tree needs to show it: its repository, its external id and the path of its node.
   *
   * @param downloads  downloads in the period, or since downloads are counted; {@code 0} for added components
   * @param created    when the component was added, as an ISO-8601 instant
   * @param createdBy  who uploaded it, if known
   * @param browsePath path of its node in the browse tree, without leading or trailing slash, if it has one
   */
  public record UsageComponentXO(
      String id,
      String repository,
      String format,
      @Nullable String group,
      String name,
      @Nullable String version,
      long downloads,
      String created,
      @Nullable String createdBy,
      @Nullable String browsePath)
  {
  }

  /**
   * A repository with its downloads in the period and what it stores.
   */
  public record RepositoryUsageXO(
      String name,
      String format,
      String type,
      long downloads,
      long storageBytes,
      long components)
  {
  }

  /**
   * The components that were not downloaded for {@code unusedDays} days and were added before that, the way cleanup
   * policies see the last download, and those that were never downloaded at all.
   *
   * @param repositories the repositories with the most unused bytes, most first
   */
  public record UnusedXO(
      long components,
      long bytes,
      long neverDownloadedComponents,
      long neverDownloadedBytes,
      List<UnusedRepositoryXO> repositories)
  {
  }

  /**
   * The unused components of one repository.
   */
  public record UnusedRepositoryXO(String name, String format, long components, long bytes)
  {
  }
}
