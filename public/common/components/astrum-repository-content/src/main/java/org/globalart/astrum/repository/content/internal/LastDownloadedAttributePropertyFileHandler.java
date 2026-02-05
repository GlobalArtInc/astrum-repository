
package org.globalart.astrum.repository.content.internal;

import java.time.OffsetDateTime;
import java.util.Optional;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.blobstore.api.BlobAttributes;
import org.globalart.astrum.blobstore.api.BlobId;
import org.globalart.astrum.blobstore.api.BlobMetrics;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.repository.MissingFacetException;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.handlers.LastDownloadedAttributeHandler;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Update the asset last downloaded time in corresponding blob's '.properties' file.
 */
@Component
public class LastDownloadedAttributePropertyFileHandler
    implements LastDownloadedAttributeHandler
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final BlobStoreManager blobStoreManager;

  @Autowired
  public LastDownloadedAttributePropertyFileHandler(final BlobStoreManager blobStoreManager) {
    this.blobStoreManager = checkNotNull(blobStoreManager);
  }

  @Override
  public void writeLastDownloadedAttribute(final FluentAsset asset) {
    FluentAsset reloadedAsset = reloadAsset(asset);
    if (reloadedAsset != null && reloadedAsset.lastDownloaded().isPresent()) {
      updateLastDownloadedPropertyIfNeeded(reloadedAsset);
    }
  }

  @Nullable
  @Override
  public OffsetDateTime readLastDownloadedAttribute(final String blobstore, final Blob blob) {
    BlobStore blobStore = blobStoreManager.get(blobstore);
    if (blobStore == null) {
      log.warn("Blob store not loaded {}", blobstore);
      return null;
    }

    BlobAttributes blobAttributes = blobStore.getBlobAttributes(blob.getId());
    if (blobAttributes == null) {
      log.warn("Blob attributes not loaded for {}", blob.getId());
      return null;
    }
    return Optional.ofNullable(blobAttributes.getMetrics())
        .map(BlobMetrics::getLastDownloaded)
        .orElse(null);
  }

  private FluentAsset reloadAsset(final FluentAsset asset) {
    try {
      ContentFacet contentFacet = asset.repository().facet(ContentFacet.class);
      return contentFacet.assets()
          .path(asset.path())
          .find()
          .orElse(null);
    }
    catch (MissingFacetException e) {
      log.warn("There is no content facet for asset {} in repository {}", asset, asset.repository());
      return null;
    }
  }

  private void updateLastDownloadedPropertyIfNeeded(final FluentAsset fluentAsset) {
    fluentAsset.blob().ifPresent(assetBlob -> {
      BlobStore blobStore = getBlobStoreForAsset(assetBlob);
      if (blobStore == null) {
        log.warn("Could not find blobstore for {}", assetBlob);
        return;
      }

      BlobId blobId = assetBlob.blobRef().getBlobId();
      BlobAttributes blobAttributes = blobStore.getBlobAttributes(blobId);
      if (blobAttributes == null) {
        log.debug("Could not get blob attributes for {}, possibly due to concurrent access", blobId);
        return;
      }

      fluentAsset.lastDownloaded().ifPresent(lastDownloaded -> {
        blobAttributes.getMetrics().setLastDownloaded(lastDownloaded);
        blobStore.setBlobAttributes(blobId, blobAttributes);
      });
    });
  }

  private BlobStore getBlobStoreForAsset(final AssetBlob assetBlob) {
    return blobStoreManager.get(assetBlob.blobRef().getStore());
  }
}
