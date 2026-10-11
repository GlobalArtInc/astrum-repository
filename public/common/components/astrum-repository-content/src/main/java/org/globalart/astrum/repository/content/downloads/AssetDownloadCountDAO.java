package org.globalart.astrum.repository.content.downloads;

import java.util.Collection;
import java.util.List;

import org.globalart.astrum.datastore.api.DataAccess;

import org.apache.ibatis.annotations.Param;

/**
 * {@link AssetDownloadCountData} access.
 */
public interface AssetDownloadCountDAO
    extends DataAccess
{
  /**
   * Adds the downloads of the given row to the stored count of its asset, creating the row when the asset has none.
   * The last downloaded time only ever moves forward.
   */
  void addDownloads(AssetDownloadCountData downloads);

  /**
   * Reads the rows of the given assets of a format; assets that were never downloaded have no row.
   */
  List<AssetDownloadCountData> browse(
      @Param("format") String format,
      @Param("assetIds") Collection<Integer> assetIds);

  /**
   * Deletes the rows of the given assets of a format.
   */
  int deleteAssets(
      @Param("format") String format,
      @Param("assetIds") Collection<Integer> assetIds);

  /**
   * Deletes the rows of the assets of the given components of a format.
   */
  int deleteComponents(
      @Param("format") String format,
      @Param("componentIds") Collection<Integer> componentIds);

  /**
   * Deletes the rows of every asset of a content repository.
   */
  int deleteRepository(
      @Param("format") String format,
      @Param("repositoryId") int repositoryId);
}
