package org.globalart.astrum.repository.content.downloads;

import java.time.LocalDate;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.List;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Stores how often the assets of each component were downloaded per day in the {@code asset_download_daily} table.
 *
 * Writes add to the stored counts, so several nodes can write the downloads they saw without coordinating.
 */
@Component
public class AssetDownloadDailyStore
    extends ConfigStoreSupport<AssetDownloadDailyDAO>
{
  private static final Comparator<AssetDownloadDailyData> LOCK_ORDER =
      Comparator.comparing(AssetDownloadDailyData::getDownloadDay)
          .thenComparing(AssetDownloadDailyData::getFormat)
          .thenComparingInt(AssetDownloadDailyData::getRepositoryId)
          .thenComparingInt(AssetDownloadDailyData::getComponentId);

  @Autowired
  public AssetDownloadDailyStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier, AssetDownloadDailyDAO.class);
  }

  /**
   * Adds the given downloads to the stored counts. Rows are written in key order so that nodes writing the same days
   * at the same time do not deadlock.
   */
  @Transactional
  public void addDownloads(final Collection<AssetDownloadDailyData> downloads) {
    List<AssetDownloadDailyData> ordered = new ArrayList<>(downloads);
    ordered.sort(LOCK_ORDER);
    AssetDownloadDailyDAO dao = dao();
    ordered.forEach(dao::addDownloads);
  }

  /**
   * Sums up the downloads of every content repository per day, from the given day on.
   */
  @Transactional
  public List<RepositoryDownloads> browseRepositoryDays(final LocalDate from) {
    return dao().browseRepositoryDays(from);
  }

  /**
   * Sums up the downloads of every content repository from the given day on, and counts the components downloaded.
   */
  @Transactional
  public List<RepositoryDownloads> browseRepositoryTotals(final LocalDate from) {
    return dao().browseRepositoryTotals(from);
  }

  /**
   * Returns the most downloaded components of the given content repositories of a format from the given day on, most
   * downloaded first.
   */
  @Transactional
  public List<ComponentDownloads> browseTopComponents(
      final LocalDate from,
      final String format,
      final Collection<Integer> repositoryIds,
      final int limit)
  {
    if (repositoryIds.isEmpty()) {
      return List.of();
    }
    return dao().browseTopComponents(from, format, repositoryIds, limit);
  }

  /**
   * Deletes the rows of the days before the given day.
   */
  @Transactional
  public int deleteBefore(final LocalDate day) {
    return dao().deleteBefore(day);
  }

  @Transactional
  public int deleteRepository(final String format, final int repositoryId) {
    return dao().deleteRepository(format, repositoryId);
  }
}
