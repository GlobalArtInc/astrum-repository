
package org.globalart.astrum.coreui.internal;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class HostedRepositoryEvaluationStateContributorTest
{
  private static final String STATE_KEY = "hostedRepositoryEvaluationEnabled";

  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    HostedRepositoryEvaluationStateContributor underTest =
        new HostedRepositoryEvaluationStateContributor(true);

    assertThat(underTest.getState().get(STATE_KEY), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    HostedRepositoryEvaluationStateContributor underTest =
        new HostedRepositoryEvaluationStateContributor(false);

    assertThat(underTest.getState().get(STATE_KEY), is(false));
  }
}
