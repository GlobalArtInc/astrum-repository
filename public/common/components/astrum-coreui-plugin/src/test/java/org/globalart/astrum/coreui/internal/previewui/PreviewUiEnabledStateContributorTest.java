
package org.globalart.astrum.coreui.internal.previewui;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class PreviewUiEnabledStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    PreviewUiEnabledStateContributor underTest = new PreviewUiEnabledStateContributor(true);

    assertThat(underTest.getState().get("astrum.previewui.enabled"), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    PreviewUiEnabledStateContributor underTest = new PreviewUiEnabledStateContributor(false);

    assertThat(underTest.getState().get("astrum.previewui.enabled"), is(false));
  }
}
