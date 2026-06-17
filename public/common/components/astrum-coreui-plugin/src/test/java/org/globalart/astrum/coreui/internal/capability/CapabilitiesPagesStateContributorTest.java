
package org.globalart.astrum.coreui.internal.capability;

import java.util.Map;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.common.app.FeatureFlags.EXTJS_CAPABILITIES_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.REACT_CAPABILITIES_ENABLED;

public class CapabilitiesPagesStateContributorTest
{
  @Test
  public void shouldReturnBothFlagsEnabled() {
    CapabilitiesPagesStateContributor underTest = new CapabilitiesPagesStateContributor(true, true);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(EXTJS_CAPABILITIES_ENABLED), is(true));
    assertThat(state.get(REACT_CAPABILITIES_ENABLED), is(true));
  }

  @Test
  public void shouldReturnBothFlagsDisabled() {
    CapabilitiesPagesStateContributor underTest = new CapabilitiesPagesStateContributor(false, false);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(EXTJS_CAPABILITIES_ENABLED), is(false));
    assertThat(state.get(REACT_CAPABILITIES_ENABLED), is(false));
  }

  @Test
  public void shouldReturnExtjsEnabledAndReactDisabled() {
    CapabilitiesPagesStateContributor underTest = new CapabilitiesPagesStateContributor(true, false);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(EXTJS_CAPABILITIES_ENABLED), is(true));
    assertThat(state.get(REACT_CAPABILITIES_ENABLED), is(false));
  }

  @Test
  public void shouldReturnExtjsDisabledAndReactEnabled() {
    CapabilitiesPagesStateContributor underTest = new CapabilitiesPagesStateContributor(false, true);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(EXTJS_CAPABILITIES_ENABLED), is(false));
    assertThat(state.get(REACT_CAPABILITIES_ENABLED), is(true));
  }
}
