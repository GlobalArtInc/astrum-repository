
package org.globalart.astrum.cleanup.internal.datastore.search.criteria;

import java.time.OffsetDateTime;
import java.util.Objects;
import java.util.function.BiPredicate;
import java.util.stream.StreamSupport;

import org.globalart.astrum.cleanup.datastore.search.criteria.ComponentCleanupEvaluator;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.Component;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_DOWNLOADED_KEY;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Tests whether all assets under a component were last downloaded before the specified offset. If an asset has never
 * been downloaded then it's blob created date is used instead of last downloaded.
 *
 */
@org.springframework.stereotype.Component
@Qualifier(LAST_DOWNLOADED_KEY)
public class LastDownloadedCleanupEvaluator
    implements ComponentCleanupEvaluator
{
  private static final Logger log = LoggerFactory.getLogger(LastDownloadedCleanupEvaluator.class);

  /*
   * Value is expected to be an offset from the current time specified in seconds.
   */
  @Override
  public BiPredicate<Component, Iterable<Asset>> getPredicate(final Repository repository, final String value) {
    OffsetDateTime cutTime = OffsetDateTime.now().minusSeconds(Long.valueOf(value));

    return (component, assets) -> {
      OffsetDateTime max = StreamSupport.stream(assets.spliterator(), false)
          .map(asset -> asset.lastDownloaded().orElse(blobCreated(asset)))
          .filter(Objects::nonNull)
          .max(OffsetDateTime::compareTo)
          .orElse(null);
      if (max != null) {
        boolean shouldCleanup = max.isBefore(cutTime);
        log.debug("{} cleanup component (assuming other criteria pass) with max last downloaded timestamp {} < {}",
            shouldCleanup ? "Should" : "Should not", max, cutTime);
        return shouldCleanup;
      }
      return false;
    };
  }

  private OffsetDateTime blobCreated(final Asset asset) {
    return asset.blob().map(AssetBlob::blobCreated).orElse(null);
  }
}
