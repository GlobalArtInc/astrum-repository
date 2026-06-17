
package org.globalart.astrum.coreui.internal.upload;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class UploadStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    UploadStateContributor underTest = new UploadStateContributor(true);

    assertThat(underTest.getState().get("astrum.react.upload"), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    UploadStateContributor underTest = new UploadStateContributor(false);

    assertThat(underTest.getState().get("astrum.react.upload"), is(false));
  }
}
