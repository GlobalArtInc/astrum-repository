
package org.globalart.astrum.coreui.internal.emailsever;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class EmailServerStateContributorTest
{
  @Test
  public void shouldReturnEnabledWhenFeatureFlagIsTrue() {
    EmailServerStateContributor underTest = new EmailServerStateContributor(true);

    assertThat(underTest.getState().get("astrum.react.emailServer"), is(true));
  }

  @Test
  public void shouldReturnDisabledWhenFeatureFlagIsFalse() {
    EmailServerStateContributor underTest = new EmailServerStateContributor(false);

    assertThat(underTest.getState().get("astrum.react.emailServer"), is(false));
  }
}
