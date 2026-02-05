
package org.globalart.astrum.repository.content.fluent;

import java.time.OffsetDateTime;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.cache.CacheInfo;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AttributeChangeSet;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;

/**
 * Fluent API for a particular asset.
 *
 * @since 3.21
 */
public interface FluentAsset
    extends Asset, FluentAttributes<FluentAsset>, FluentAssetBlobAttach
{
  /**
   * The repository containing this asset.
   *
   * @since 3.24
   */
  Repository repository();

  /**
   * Downloads this asset.
   */
  Content download();

  /**
   * Mark this asset as recently downloaded by a user action. Throttled by the configured
   * lastDownloaded interval — repeated calls within the interval are no-ops at the database.
   */
  FluentAsset markAsDownloaded();

  /**
   * Variant of {@link #markAsDownloaded()} that returns whether the database row was actually
   * updated. Use this when you need to gate follow-up work (such as updating blob attribute
   * files) on the throttle outcome — {@code false} indicates another caller updated within the
   * interval and the file write should be skipped.
   *
   * @return {@code true} if the database row was updated; {@code false} if the throttle skipped
   *         the update
   */
  boolean tryMarkAsDownloaded();

  /**
   * Mark this proxied asset as recently cached from the given content.
   *
   * @since 3.25
   */
  FluentAsset markAsCached(Payload content);

  /**
   * Mark this generated asset as recently cached.
   *
   * @since 3.24
   */
  FluentAsset markAsCached(CacheInfo cacheInfo);

  /**
   * Mark this generated/proxied asset as stale.
   *
   * @since 3.24
   */
  FluentAsset markAsStale();

  /**
   * Is this generated/proxied asset stale?
   *
   * @since 3.24
   */
  boolean isStale(CacheController cacheController);

  /**
   * Update this asset to have the given kind.
   *
   * @since 3.25
   */
  FluentAsset kind(String kind);

  /**
   * Generally it is recommended that this method not be called and let stores manage this value automatically.
   * Update this asset to have the given path.
   */
  FluentAsset updatePath(String path);

  /**
   * Deletes this asset.
   */
  boolean delete();

  /**
   * Generally it is recommended that this method not be called and let stores manage this value.
   *
   * @since 3.29
   */
  void blobCreated(OffsetDateTime blobCreated);

  /**
   * Sets added to repository on the asset blob.
   */
  void blobAddedToRepository(final OffsetDateTime addedToRepository);

  /**
   * Generally it is recommended that this method not be called and let stores manage this value automatically.
   *
   * Sets the created time of the asset associated with the ID to the specified time.
   *
   * @since 3.29
   */
  void created(OffsetDateTime created);

  /**
   * Generally it is recommended that this method not be called and let stores manage this value automatically.
   *
   * Sets the last download time of the asset associated with the ID to the specified time.
   *
   * @since 3.29
   */
  void lastDownloaded(OffsetDateTime lastDownloaded);

  /**
   * Generally it is recommended that this method not be called and let stores manage this value automatically.
   *
   * Sets the last updated time of the asset associated with the ID to the specified time.
   *
   * @since 3.29
   */
  void lastUpdated(OffsetDateTime lastUpdated);

  /**
   * Applies the given change to the current attributes.
   */
  FluentAsset attributes(AttributeChangeSet changeset);

  /**
   * Generally it is recommended that this method not be called and let stores manage this value automatically.
   *
   * Sets the name of user who uploaded the asset.
   */
  void createdBy(String createdBy);

  /**
   * Generally it is recommended that this method not be called and let stores manage this value automatically.
   *
   * Sets the IP addres of user who uploaded the asset.
   */
  void createdByIP(String createdByIP);
}
