
package org.globalart.astrum.coreui.internal;

import java.util.Map;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.common.app.FeatureFlags.CLUSTERED_ZERO_DOWNTIME_ENABLED;
import static org.globalart.astrum.common.app.FeatureFlags.DATASTORE_CLUSTERED_ENABLED;

public class ClusteredModeStateContributorTest
{
  @Test
  public void shouldReturnBothFlagsEnabled() {
    ClusteredModeStateContributor underTest = new ClusteredModeStateContributor(true, true);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(DATASTORE_CLUSTERED_ENABLED), is(true));
    assertThat(state.get(CLUSTERED_ZERO_DOWNTIME_ENABLED), is(true));
  }

  @Test
  public void shouldReturnBothFlagsDisabled() {
    ClusteredModeStateContributor underTest = new ClusteredModeStateContributor(false, false);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(DATASTORE_CLUSTERED_ENABLED), is(false));
    assertThat(state.get(CLUSTERED_ZERO_DOWNTIME_ENABLED), is(false));
  }

  @Test
  public void shouldReturnClusteredEnabledAndZeroDowntimeDisabled() {
    ClusteredModeStateContributor underTest = new ClusteredModeStateContributor(true, false);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(DATASTORE_CLUSTERED_ENABLED), is(true));
    assertThat(state.get(CLUSTERED_ZERO_DOWNTIME_ENABLED), is(false));
  }

  @Test
  public void shouldReturnClusteredDisabledAndZeroDowntimeEnabled() {
    ClusteredModeStateContributor underTest = new ClusteredModeStateContributor(false, true);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get(DATASTORE_CLUSTERED_ENABLED), is(false));
    assertThat(state.get(CLUSTERED_ZERO_DOWNTIME_ENABLED), is(true));
  }
}
