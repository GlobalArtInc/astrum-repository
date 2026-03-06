
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
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

class LastBlobUpdatedCleanupEvaluatorTest
    extends LastBaseCleanupEvaluatorTestSupport
{

  private LastBlobUpdatedCleanupEvaluator underTest;

  @Override
  protected void setupMocks() {
    underTest = new LastBlobUpdatedCleanupEvaluator();
  }

  @Test
  void testPredicateShouldCleanupWhenBlobUpdatedBeforeCutTime() {
    OffsetDateTime now = OffsetDateTime.now();
    OffsetDateTime oldBlobTime = now.minusDays(2);
    String value = String.valueOf(OffsetDateTime.now().toEpochSecond() - oldBlobTime.toEpochSecond());

    Asset asset = createMockAssetWithBlob(oldBlobTime);
    Component component = createMockComponent();

    BiPredicate<Component, Iterable<Asset>> predicate = underTest.getPredicate(repository, value);
    boolean result = predicate.test(component, List.of(asset));

    assertThat(result, is(true));
  }

  @Test
  void testPredicateShouldNotCleanupWhenBlobUpdatedAfterCutTime() {
    OffsetDateTime now = OffsetDateTime.now();
    OffsetDateTime recentBlobTime = now.minusHours(1);
    String value = "86400"; // 1 day offset

    Asset asset = createMockAssetWithBlob(recentBlobTime);
    Component component = createMockComponent();

    BiPredicate<Component, Iterable<Asset>> predicate = underTest.getPredicate(repository, value);
    boolean result = predicate.test(component, List.of(asset));

    assertThat(result, is(false));
  }

  @Test
  void testPredicateShouldNotCleanupWhenNoBlobPresent() {
    String value = "86400"; // 1 day offset

    Asset asset = mock(Asset.class);
    when(asset.blob()).thenReturn(Optional.empty());

    Component component = createMockComponent();

    BiPredicate<Component, Iterable<Asset>> predicate = underTest.getPredicate(repository, value);
    boolean result = predicate.test(component, List.of(asset));

    assertThat(result, is(false));
  }
}
