
package org.globalart.astrum.repository.content.store;

import java.time.OffsetDateTime;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.ContinuationAware;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.Component;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Optional.ofNullable;
import static org.globalart.astrum.repository.content.store.InternalIds.internalAssetBlobId;
import static org.globalart.astrum.repository.content.store.InternalIds.internalComponentId;

/**
 * {@link Asset} data backed by the content data store.
 *
 * @since 3.20
 */
public class AssetData
    extends AbstractRepositoryContent
    implements Asset, ContinuationAware
{
  Integer assetId; // NOSONAR: internal id

  private String path;

  private String kind;

  @Nullable
  Integer componentId; // NOSONAR: internal id

  @Nullable
  private Component component;

  @Nullable
  Integer assetBlobId; // NOSONAR: internal id

  @Nullable
  private AssetBlob assetBlob;

  @Nullable
  private OffsetDateTime lastDownloaded;

  @Nullable
  private String blobStoreName;

  @Nullable
  private long assetBlobSize;

  /**
   * Flag to track whether this asset was retrieved from an eager query (browseEager).
   * Used to determine the appropriate continuation token format.
   */
  private boolean fromEagerQuery = false;

  // Asset API

  @Override
  public String path() {
    return path;
  }

  @Override
  public String kind() {
    return kind;
  }

  @Override
  public Optional<Component> component() {
    return ofNullable(getComponent()); // trigger lazy-loading by calling getter
  }

  @Override
  public Optional<AssetBlob> blob() {
    return ofNullable(getAssetBlob()); // trigger lazy-loading by calling getter
  }

  @Override
  public Optional<OffsetDateTime> lastDownloaded() {
    return ofNullable(lastDownloaded);
  }

  @Override
  public String blobStoreName() {
    return blobStoreName;
  }

  @Override
  public long assetBlobSize() {
    return assetBlobSize;
  }

  // MyBatis setters + validation

  /**
   * Sets the blob store name.
   */
  public void setBlobStoreName(@Nullable final String blobStoreName) {
    this.blobStoreName = blobStoreName;
  }

  /**
   * Sets the internal asset id.
   */
  public void setAssetId(final int assetId) {
    this.assetId = assetId;
  }

  /**
   * Set the internal reference to the blob
   */
  public void setAssetBlobId(final Integer assetBlobId) {
    this.assetBlobId = assetBlobId;
  }

  /**
   * Sets the asset path; asset paths must start with a slash.
   */
  public void setPath(final String path) {
    checkArgument(path != null && path.charAt(0) == '/', "Paths must start with a slash");
    this.path = path;
  }

  /**
   * Sets the asset kind.
   *
   * @since 3.24
   */
  public void setKind(final String kind) {
    this.kind = checkNotNull(kind);
  }

  /**
   * Sets the (optional) owning component.
   */
  public void setComponent(@Nullable final Component component) {
    if (component != null) {
      this.componentId = internalComponentId(component);
    }
    else {
      this.componentId = null;
    }
    this.component = component;
  }

  /**
   * Sets the (optional) blob attached to this asset.
   */
  public void setAssetBlob(@Nullable final AssetBlob assetBlob) {
    if (assetBlob != null) {
      this.assetBlobId = internalAssetBlobId(assetBlob);
    }
    else {
      this.assetBlobId = null;
    }
    this.assetBlob = assetBlob;
  }

  @Override
  public boolean hasBlob() {
    return assetBlobId != null;
  }

  /**
   * Sets the (optional) last downloaded time.
   */
  public void setLastDownloaded(@Nullable final OffsetDateTime lastDownloaded) {
    this.lastDownloaded = lastDownloaded;
  }

  public void setAssetBlobSize(final long assetBlobSize) {
    this.assetBlobSize = assetBlobSize;
  }

  /**
   * Sets whether this asset was retrieved from an eager query.
   * This is set by MyBatis for assets from browseEagerAssetsInRepository queries.
   */
  public void setFromEagerQuery(final boolean fromEagerQuery) {
    this.fromEagerQuery = fromEagerQuery;
  }

  // Getters to support lazy-loading (MyBatis will intercept them)

  @Nullable
  protected Component getComponent() {
    return component;
  }

  @Nullable
  protected AssetBlob getAssetBlob() {
    return assetBlob;
  }

  // ContinuationAware

  @Override
  public String nextContinuationToken() {
    // Use explicit flag to determine token format
    // For browseEagerAssetsInRepository queries, ALWAYS use composite token for consistency
    // For other queries, use simple asset_id token
    if (fromEagerQuery) {
      long blobCreatedMillis = 0L;
      if (assetBlob != null && assetBlob.blobCreated() != null) {
        blobCreatedMillis = assetBlob.blobCreated().toInstant().toEpochMilli();
      }
      return new AssetBlobCreatedContinuationToken(blobCreatedMillis, assetId).encode();
    }
    return Integer.toString(assetId);
  }

  @Override
  public String toString() {
    return "AssetData{" +
        "assetId=" + assetId +
        ", path='" + path + '\'' +
        ", kind='" + kind + '\'' +
        ", componentId=" + componentId +
        ", component=" + component +
        ", assetBlobId=" + assetBlobId +
        ", assetBlob=" + assetBlob +
        ", lastDownloaded=" + lastDownloaded +
        ", assetSize=" + assetBlobSize +
        "} " + super.toString();
  }
}
