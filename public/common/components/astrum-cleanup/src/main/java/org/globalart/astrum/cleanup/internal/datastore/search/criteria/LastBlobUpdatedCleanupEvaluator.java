
package org.globalart.astrum.cleanup.internal.datastore.search.criteria;

import java.time.OffsetDateTime;
import java.util.Optional;
import java.util.function.BiPredicate;
import java.util.stream.StreamSupport;

import org.globalart.astrum.cleanup.datastore.search.criteria.ComponentCleanupEvaluator;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.Component;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.globalart.astrum.cleanup.config.CleanupPolicyConstants.LAST_BLOB_UPDATED_KEY;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Creates a test which evaluates whether all assets under a component had their blob created (i.e. attached) before the
 * time determined by the offset.
 *
 */
@org.springframework.stereotype.Component
@Qualifier(LAST_BLOB_UPDATED_KEY)
public class LastBlobUpdatedCleanupEvaluator
    implements ComponentCleanupEvaluator
{
  private static final Logger log = LoggerFactory.getLogger(LastBlobUpdatedCleanupEvaluator.class);

  /*
   * Value is expected to be an offset from the current time specified in seconds.
   */
  @Override
  public BiPredicate<Component, Iterable<Asset>> getPredicate(final Repository repository, final String value) {
    OffsetDateTime cutTime = OffsetDateTime.now().minusSeconds(Long.valueOf(value));

    return (component, assets) -> {
      OffsetDateTime max = StreamSupport.stream(assets.spliterator(), false)
          .map(Asset::blob)
          .filter(Optional::isPresent)
          .map(Optional::get)
          .map(AssetBlob::blobCreated)
          .max(OffsetDateTime::compareTo)
          .orElse(null);
      if (max != null) {
        boolean shouldCleanup = max.isBefore(cutTime);
        log.debug("{} cleanup component (assuming other criteria pass) with max blob updated timestamp {} < {}",
            shouldCleanup ? "Should" : "Should not", max, cutTime);
        return shouldCleanup;
      }
      return false;
    };
  }
}
