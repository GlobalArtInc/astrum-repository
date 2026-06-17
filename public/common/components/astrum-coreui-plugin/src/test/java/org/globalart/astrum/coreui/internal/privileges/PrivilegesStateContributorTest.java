
package org.globalart.astrum.coreui.internal.privileges;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_PRIVILEGES;

public class PrivilegesStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    PrivilegesStateContributor underTest = new PrivilegesStateContributor(true);

    assertThat(underTest.getState().get(REACT_PRIVILEGES), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    PrivilegesStateContributor underTest = new PrivilegesStateContributor(false);

    assertThat(underTest.getState().get(REACT_PRIVILEGES), is(false));
  }
}
