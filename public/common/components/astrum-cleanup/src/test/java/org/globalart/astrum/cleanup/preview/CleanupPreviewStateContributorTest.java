
package org.globalart.astrum.cleanup.preview;

import java.util.Map;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.globalart.astrum.common.app.FeatureFlags.CLEANUP_PREVIEW_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.CLEANUP_RETAIN_ALL_FORMATS;

public class CleanupPreviewStateContributorTest
{
  @Test
  public void testGetStateBothFlagsTrue() {
    CleanupPreviewStateContributor underTest = new CleanupPreviewStateContributor(true, true);

    Map<String, Object> state = underTest.getState();

    assertThat(state).containsEntry(CLEANUP_PREVIEW_ENABLED, true);
    assertThat(state).containsEntry(CLEANUP_RETAIN_ALL_FORMATS, true);
  }

  @Test
  public void testGetStateBothFlagsFalse() {
    CleanupPreviewStateContributor underTest = new CleanupPreviewStateContributor(false, false);

    Map<String, Object> state = underTest.getState();

    assertThat(state).containsEntry(CLEANUP_PREVIEW_ENABLED, false);
    assertThat(state).containsEntry(CLEANUP_RETAIN_ALL_FORMATS, false);
  }

  @Test
  public void testGetStateMixedFlags() {
    CleanupPreviewStateContributor underTest = new CleanupPreviewStateContributor(true, false);

    Map<String, Object> state = underTest.getState();

    assertThat(state).containsEntry(CLEANUP_PREVIEW_ENABLED, true);
    assertThat(state).containsEntry(CLEANUP_RETAIN_ALL_FORMATS, false);
  }

  @Test
  public void testGetStateContainsOnlyExpectedKeys() {
    CleanupPreviewStateContributor underTest = new CleanupPreviewStateContributor(true, true);

    Map<String, Object> state = underTest.getState();

    assertThat(state).hasSize(2);
    assertThat(state).containsOnlyKeys(CLEANUP_PREVIEW_ENABLED, CLEANUP_RETAIN_ALL_FORMATS);
  }
}
