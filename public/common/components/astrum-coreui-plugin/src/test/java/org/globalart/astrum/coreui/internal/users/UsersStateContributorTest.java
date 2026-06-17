
package org.globalart.astrum.coreui.internal.users;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class UsersStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    UsersStateContributor underTest = new UsersStateContributor(true);

    assertThat(underTest.getState().get("astrum.react.users"), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    UsersStateContributor underTest = new UsersStateContributor(false);

    assertThat(underTest.getState().get("astrum.react.users"), is(false));
  }
}
