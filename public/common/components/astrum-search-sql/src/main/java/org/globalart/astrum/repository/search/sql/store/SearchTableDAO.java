
package org.globalart.astrum.repository.search.sql.store;

import java.util.Collection;
import java.util.List;
import java.util.Set;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.search.sql.query.SqlSearchRequest;

import org.globalart.astrum.datastore.api.ContentDataAccess;
import org.globalart.astrum.repository.search.sql.SearchAssetRecord;
import org.globalart.astrum.repository.search.sql.SearchResult;

import org.apache.ibatis.annotations.Param;

/**
 * DAO for access search table entries
 */
public interface SearchTableDAO
    extends ContentDataAccess
{
  /**
   * Count components in the given format.
   *
   * @param request DTO containing all required params for search
   * @return number of found components.
   */
  long count(SqlSearchRequest request);

  /**
   * Search components in the scope of one format.
   *
   * @param request DTO containing all required params for search
   * @return collection of {@link SearchResultData} representing search results for a given format.
   */
  Collection<SearchResult> searchComponents(SqlSearchRequest request);

  /**
   * Saves the given search entry in the content data store by performing an upsert.
   *
   * @param recordData the search row to create
   */
  void save(SearchRecordData recordData);

  /**
   * Delete the given search entry in the content data store.
   *
   * @param repositoryId the content repository identification
   * @param componentId the component identification
   * @param format the repository format
   */
  void delete(
      @Param("repositoryId") Integer repositoryId,
      @Param("componentId") Integer componentId,
      @Param("format") String format);

  /**
   * Delete records for the specified repository, format and component ids.
   *
   * @param repositoryId the content repository id
   * @param componentIds the component ids to delete
   * @param format the format
   */
  void deleteComponentIds(
      @Param("repositoryId") Integer repositoryId,
      @Param("componentIds") Set<Integer> componentIds,
      @Param("format") String format);

  /**
   * Delete asset records for the specified repository, format and component ids.
   *
   * @param repositoryId the content repository id
   * @param componentIds the component ids to delete
   * @param format the format
   */
  void deleteSearchAssets(
      @Param("repositoryId") Integer repositoryId,
      @Nonnull @Param("componentIds") final Set<Integer> componentIds,
      @Param("format") String format);

  /**
   * Delete all search entries for given repository.
   *
   * @param repositoryId the content repository identification
   * @param format the repository format
   * @param limit when positive limits the number of entries deleted per-call
   * @return {@code true} if any record was deleted
   */
  boolean deleteAllForRepository(
      @Param("repositoryId") Integer repositoryId,
      @Param("format") String format,
      @Param("limit") int limit);

  /**
   * Delete all search_asset entries for given repository.
   *
   * @param repositoryId the content repository identification
   * @param format the repository format
   * @param limit when positive limits the number of entries deleted per-call
   * @return {@code true} if any record was deleted
   */
  boolean deleteAllSearchAssets(
      @Param("repositoryId") Integer repositoryId,
      @Param("format") String format,
      @Param("limit") int limit);

  /**
   * Delete search_components entries whose component_id no longer exists in the
   * format-specific component table. Used after a rebuild to remove records for
   * deleted components. Uses NOT EXISTS rather than last_modified timestamps to
   * avoid false positives from old blob.blobCreated() timestamps.
   *
   * @param repositoryId the content repository identification
   * @param format the repository format
   * @param limit when positive limits the number of entries deleted per-call
   * @return {@code true} if any record was deleted
   */
  boolean deleteOrphanedComponents(
      @Param("repositoryId") Integer repositoryId,
      @Param("format") String format,
      @Param("limit") int limit);

  /**
   * Delete search_assets entries for a repository whose component_id no longer exists in
   * search_components. Used after a rebuild to remove asset records for deleted components.
   *
   * @param repositoryId the content repository identification
   * @param format the repository format
   * @param limit when positive limits the number of entries deleted per-call
   * @return {@code true} if any record was deleted
   */
  boolean deleteOrphanedAssets(
      @Param("repositoryId") Integer repositoryId,
      @Param("format") String format,
      @Param("limit") int limit);

  /**
   * Batch Insert data.
   *
   * @param records data to be saved.
   */
  void saveBatch(@Param("searchData") List<SearchRecordData> records);

  /**
   * Batch Insert assets data.
   *
   * @param searchAssetRecords data to be saved.
   */
  void saveAssets(@Param("searchAssetRecords") Collection<SearchAssetRecord> searchAssetRecords);

  /**
   * Saves the asset records if that asset still exists
   */
  void saveAsset(@Param("searchAssetRecord") SearchAssetRecord saveAssetRecord);

  /**
   * Check repository has search entries.
   *
   * @param repositoryName repository
   * @return {@code true} if any records exists
   */
  boolean hasRepositoryEntries(@Param("repositoryName") final String repositoryName);

  /**
   * Get all the unique repositories in the search tables.
   *
   * @return list of unique repositories wrapped in {@link SearchRepositoryData}
   */
  List<SearchRepositoryData> getSearchRepositories();

  /**
   * Browses distinct versions of a component, newest first, with the repositories containing each.
   *
   * @param request the filter, sort, and page bounds
   * @return one entry per distinct version
   */
  List<ComponentVersionData> browseComponentVersions(ComponentVersionSearchRequest request);

  /**
   * Counts distinct versions matching the filter. Unlike {@link #count}, this does not
   * over-count components that span multiple repositories.
   *
   * @param request the filter (sort and page bounds are ignored)
   * @return the number of distinct versions
   */
  long countComponentVersions(ComponentVersionSearchRequest request);
}
