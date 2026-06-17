
package org.globalart.astrum.coreui;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class ApiDocsStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    ApiDocsStateContributor underTest = new ApiDocsStateContributor(true);

    assertThat(underTest.getState().get("api"), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    ApiDocsStateContributor underTest = new ApiDocsStateContributor(false);

    assertThat(underTest.getState().get("api"), is(false));
  }
}
