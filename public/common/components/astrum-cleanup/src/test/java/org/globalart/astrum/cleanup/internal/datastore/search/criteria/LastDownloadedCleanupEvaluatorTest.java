
package org.globalart.astrum.cleanup.internal.datastore.search.criteria;

import java.time.OffsetDateTime;
import java.util.List;
import java.util.Optional;
import java.util.function.BiPredicate;

import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.Component;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.when;

class LastDownloadedCleanupEvaluatorTest
    extends LastBaseCleanupEvaluatorTestSupport
{

  private LastDownloadedCleanupEvaluator underTest;

  @Override
  protected void setupMocks() {
    underTest = new LastDownloadedCleanupEvaluator();
  }

  @Test
  void testPredicateShouldCleanupWhenLastDownloadedBeforeCutTime() {
    OffsetDateTime now = OffsetDateTime.now();
    OffsetDateTime oldDownloadTime = now.minusDays(2);
    // Use the same 'now' reference to avoid timing drift between two OffsetDateTime.now() calls
    String value = String.valueOf(now.toEpochSecond() - oldDownloadTime.toEpochSecond());

    Asset asset = createMockAssetWithLastDownloaded(oldDownloadTime);
    Component component = createMockComponent();

    BiPredicate<Component, Iterable<Asset>> predicate = underTest.getPredicate(repository, value);
    boolean result = predicate.test(component, List.of(asset));

    assertThat(result, is(true));
  }

  @Test
  void testPredicateShouldNotCleanupWhenLastDownloadedAfterCutTime() {
    OffsetDateTime now = OffsetDateTime.now();
    OffsetDateTime recentDownloadTime = now.minusHours(1);
    String value = "86400"; // 1 day offset

    Asset asset = createMockAssetWithLastDownloaded(recentDownloadTime);
    Component component = createMockComponent();

    BiPredicate<Component, Iterable<Asset>> predicate = underTest.getPredicate(repository, value);
    boolean result = predicate.test(component, List.of(asset));

    assertThat(result, is(false));
  }

  @Test
  void testPredicateShouldUseBlobCreatedIfNeverDownloaded() {
    OffsetDateTime now = OffsetDateTime.now();
    OffsetDateTime oldBlobTime = now.minusDays(2);
    String value = "86400"; // 1 day offset

    Asset asset = createMockAssetWithBlob(oldBlobTime);
    when(asset.lastDownloaded()).thenReturn(Optional.empty());

    Component component = createMockComponent();

    BiPredicate<Component, Iterable<Asset>> predicate = underTest.getPredicate(repository, value);
    boolean result = predicate.test(component, List.of(asset));

    assertThat(result, is(true));
  }
}
