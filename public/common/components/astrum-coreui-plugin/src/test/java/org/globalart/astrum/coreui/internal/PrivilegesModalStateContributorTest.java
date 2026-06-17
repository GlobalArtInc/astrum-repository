
package org.globalart.astrum.coreui.internal;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_PRIVILEGES_MODAL_ENABLED;

public class PrivilegesModalStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    PrivilegesModalStateContributor underTest = new PrivilegesModalStateContributor(true);

    assertThat(underTest.getState().get(REACT_PRIVILEGES_MODAL_ENABLED), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    PrivilegesModalStateContributor underTest = new PrivilegesModalStateContributor(false);

    assertThat(underTest.getState().get(REACT_PRIVILEGES_MODAL_ENABLED), is(false));
  }
}
