
package org.globalart.astrum.coreui.internal;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_ROLES_MODAL_ENABLED;

public class RolesModalStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    RolesModalStateContributor underTest = new RolesModalStateContributor(true);

    assertThat(underTest.getState().get(REACT_ROLES_MODAL_ENABLED), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    RolesModalStateContributor underTest = new RolesModalStateContributor(false);

    assertThat(underTest.getState().get(REACT_ROLES_MODAL_ENABLED), is(false));
  }
}
