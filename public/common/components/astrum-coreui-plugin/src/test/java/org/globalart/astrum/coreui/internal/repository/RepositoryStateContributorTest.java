
package org.globalart.astrum.coreui.internal.repository;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class RepositoryStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    RepositoryStateContributor underTest = new RepositoryStateContributor(true);

    assertThat(underTest.getState().get("astrum.react.repositories"), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    RepositoryStateContributor underTest = new RepositoryStateContributor(false);

    assertThat(underTest.getState().get("astrum.react.repositories"), is(false));
  }
}
