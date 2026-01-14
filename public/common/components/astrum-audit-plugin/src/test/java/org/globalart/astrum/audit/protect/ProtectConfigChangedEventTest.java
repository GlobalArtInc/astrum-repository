
package org.globalart.astrum.audit.protect;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

public class ProtectConfigChangedEventTest
{
  @Test
  public void testEventProperties() {
    ProtectConfigChangedEvent event = new ProtectConfigChangedEvent(
        "protect.firewall", "protection-level-changed",
        "maven-proxy", "none", "quarantine");

    assertThat(event.getDomain(), is("protect.firewall"));
    assertThat(event.getType(), is("protection-level-changed"));
    assertThat(event.getContext(), is("maven-proxy"));
    assertThat(event.getFromValue(), is("none"));
    assertThat(event.getToValue(), is("quarantine"));
  }
}
