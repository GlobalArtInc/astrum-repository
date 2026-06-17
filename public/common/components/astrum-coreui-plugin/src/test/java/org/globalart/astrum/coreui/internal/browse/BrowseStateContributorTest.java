
package org.globalart.astrum.coreui.internal.browse;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class BrowseStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    BrowseStateContributor underTest = new BrowseStateContributor(true);

    assertThat(underTest.getState().get("astrum.react.browse"), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    BrowseStateContributor underTest = new BrowseStateContributor(false);

    assertThat(underTest.getState().get("astrum.react.browse"), is(false));
  }
}
