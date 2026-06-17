
package org.globalart.astrum.coreui.internal.log;

import java.util.Map;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class LogViewerStateContributorTest
{
  @Test
  public void shouldReturnLogViewerEnabledWhenNotClustered() {
    LogViewerStateContributor underTest = new LogViewerStateContributor(false);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get("log.viewer.enabled"), is(true));
  }

  @Test
  public void shouldReturnLogViewerDisabledWhenClustered() {
    LogViewerStateContributor underTest = new LogViewerStateContributor(true);

    Map<String, Object> state = underTest.getState();

    assertThat(state.get("log.viewer.enabled"), is(false));
  }
}
