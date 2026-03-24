
package org.globalart.astrum.repository.maven.internal.content.importtask;

import java.time.OffsetDateTime;
import java.util.Optional;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.importtask.ImportPostProcessor;
import org.globalart.astrum.repository.maven.MavenPath.HashType;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Provides some post processing capabilities for assets that could have hash files that need blobUpdated date to match
 * the imported asset blobUpdated Date
 *
 * @since 3.29
 */
@Component
@Qualifier(Maven2Format.NAME)
public class MavenImportPostProcessor
    implements ImportPostProcessor
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  @Override
  public void attributePostProcessing(
      final Repository repository,
      final Asset asset)
  {
    for (HashType type : HashType.values()) {
      updateHashFile('.' + type.getExt(), asset, repository);
    }
  }

  private void updateHashFile(
      final String hashFileExtension,
      final Asset asset,
      final Repository repository)
  {
    Optional<FluentAsset> hashAsset = repository.facet(ContentFacet.class)
        .assets()
        .path(asset.path() + hashFileExtension)
        .find();

    if (hashAsset.isPresent()) {
      updateHashFileDates(hashAsset.get(), asset);
    }
  }

  private void updateHashFileDates(final FluentAsset hashAsset, final Asset asset) {
    Optional<OffsetDateTime> lastDownloaded = hashAsset.lastDownloaded();
    if (!lastDownloaded.isPresent() ||
        !lastDownloaded.get().equals(asset.lastDownloaded().orElse(null))) {
      hashAsset.lastDownloaded(asset.lastDownloaded().orElse(null));
    }
    if (!hashAsset.created().equals(asset.created())) {
      hashAsset.created(asset.created());
    }
    if (hashAsset.lastUpdated() == null ||
        (hashAsset.lastUpdated() != null && !hashAsset.lastUpdated().equals(asset.lastUpdated()))) {
      hashAsset.lastUpdated(asset.lastUpdated());
    }

    hashAsset.blob().ifPresent(blob -> {
      asset.blob().ifPresent(assetBlob -> {
        if (blob.blobCreated() == null ||
            (blob.blobCreated() != null && !blob.blobCreated().equals(assetBlob.blobCreated()))) {
          hashAsset.blobCreated(assetBlob.blobCreated());
        }
      });
    });
  }
}
