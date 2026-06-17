
package org.globalart.astrum.coreui.internal.tasks;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class TasksStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    TasksStateContributor underTest = new TasksStateContributor(true);

    assertThat(underTest.getState().get("astrum.react.tasks"), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    TasksStateContributor underTest = new TasksStateContributor(false);

    assertThat(underTest.getState().get("astrum.react.tasks"), is(false));
  }
}
