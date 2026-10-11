package org.globalart.astrum.repository.content.downloads;

import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
import java.util.List;

import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import com.google.common.collect.Iterables;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

/**
 * Stores the download counts of assets in the {@code asset_download_count} table.
 *
 * Writes add to the stored counts, so several nodes can write the downloads they saw without coordinating.
 */
@Component
public class AssetDownloadCountStore
    extends ConfigStoreSupport<AssetDownloadCountDAO>
{
  static final int BATCH_SIZE = 1000;

  private static final Comparator<AssetDownloadCountData> LOCK_ORDER =
      Comparator.comparing(AssetDownloadCountData::getFormat).thenComparingInt(AssetDownloadCountData::getAssetId);

  @Autowired
  public AssetDownloadCountStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier, AssetDownloadCountDAO.class);
  }

  /**
   * Adds the given downloads to the stored counts. Rows are written in key order so that nodes writing the same
   * assets at the same time do not deadlock.
   */
  @Transactional
  public void addDownloads(final Collection<AssetDownloadCountData> downloads) {
    List<AssetDownloadCountData> ordered = new ArrayList<>(downloads);
    ordered.sort(LOCK_ORDER);
    AssetDownloadCountDAO dao = dao();
    ordered.forEach(dao::addDownloads);
  }

  /**
   * Reads the stored rows of the given assets of a format; assets that were never downloaded have no row.
   */
  @Transactional
  public List<AssetDownloadCountData> browse(final String format, final Collection<Integer> assetIds) {
    List<AssetDownloadCountData> rows = new ArrayList<>();
    for (List<Integer> batch : Iterables.partition(assetIds, BATCH_SIZE)) {
      rows.addAll(dao().browse(format, batch));
    }
    return rows;
  }

  @Transactional
  public int deleteAssets(final String format, final Collection<Integer> assetIds) {
    int deleted = 0;
    for (List<Integer> batch : Iterables.partition(assetIds, BATCH_SIZE)) {
      deleted += dao().deleteAssets(format, batch);
    }
    return deleted;
  }

  @Transactional
  public int deleteComponents(final String format, final Collection<Integer> componentIds) {
    int deleted = 0;
    for (List<Integer> batch : Iterables.partition(componentIds, BATCH_SIZE)) {
      deleted += dao().deleteComponents(format, batch);
    }
    return deleted;
  }

  @Transactional
  public int deleteRepository(final String format, final int repositoryId) {
    return dao().deleteRepository(format, repositoryId);
  }
}
