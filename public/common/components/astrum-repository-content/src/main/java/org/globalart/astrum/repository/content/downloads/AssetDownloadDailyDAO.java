package org.globalart.astrum.repository.content.downloads;

import java.time.LocalDate;
import java.util.Collection;
import java.util.List;

import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.annotations.Param;

/**
 * {@link AssetDownloadDailyData} access.
 */
public interface AssetDownloadDailyDAO
    extends DataAccess
{
  /**
   * Adds the downloads of the given row to the stored count of its day and component, creating the row when there is
   * none.
   */
  void addDownloads(AssetDownloadDailyData downloads);

  /**
   * Sums up the downloads of every content repository per day, from the given day on.
   */
  List<RepositoryDownloads> browseRepositoryDays(@Param("from") LocalDate from);

  /**
   * Sums up the downloads of every content repository from the given day on, and counts the components downloaded.
   */
  List<RepositoryDownloads> browseRepositoryTotals(@Param("from") LocalDate from);

  /**
   * Returns the most downloaded components of the given content repositories of a format from the given day on, most
   * downloaded first.
   */
  List<ComponentDownloads> browseTopComponents(
      @Param("from") LocalDate from,
      @Param("format") String format,
      @Param("repositoryIds") Collection<Integer> repositoryIds,
      @Param("limit") int limit);

  /**
   * Deletes the rows of the days before the given day.
   */
  int deleteBefore(@Param("day") LocalDate day);

  /**
   * Deletes the rows of a content repository.
   */
  int deleteRepository(
      @Param("format") String format,
      @Param("repositoryId") int repositoryId);
}
