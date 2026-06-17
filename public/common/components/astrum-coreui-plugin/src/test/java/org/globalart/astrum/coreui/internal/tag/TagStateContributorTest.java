
package org.globalart.astrum.coreui.internal.tag;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class TagStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    TagStateContributor underTest = new TagStateContributor(true);

    assertThat(underTest.getState().get("astrum.react.tags"), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    TagStateContributor underTest = new TagStateContributor(false);

    assertThat(underTest.getState().get("astrum.react.tags"), is(false));
  }
}
