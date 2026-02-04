
package org.globalart.astrum.repository.content;

import java.time.OffsetDateTime;
import java.util.Optional;

/**
 * Each asset represents a unique path to binary content in a repository.
 *
 * @since 3.20
 * @see Component
 */
public interface Asset
    extends RepositoryContent
{
  /**
   * The path in the repository.
   */
  String path();

  /**
   * The kind of asset.
   *
   * @since 3.24
   */
  String kind();

  /**
   * Assets may be grouped together under a logical coordinate, represented by a {@link Component}.
   */
  Optional<Component> component();

  /**
   * Current blob attached to this asset; proxy repositories may have assets whose blobs have not been fetched yet.
   * If checking for existence please use {@code hasBlob()} which is less expensive.
   */
  Optional<AssetBlob> blob();

  /**
   * Indicates whether the asset has a blob, may be faster than {@code blob()} due to lazy loading.
   */
  boolean hasBlob();

  /**
   * If/when this asset was last downloaded.
   */
  Optional<OffsetDateTime> lastDownloaded();

  /**
   * returns the blob store name if blob_store_name is in the query
   */
  String blobStoreName();

  /**
   * The size of the asset(blob)
   */
  public long assetBlobSize();
}
