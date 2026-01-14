
package org.globalart.astrum.audit.protect;

import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;
import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ProtectConfigAuditorTest
{
  @Mock
  private AuditRecorder auditRecorder;

  @Captor
  private ArgumentCaptor<AuditData> auditDataCaptor;

  private ProtectConfigAuditor auditor;

  @Before
  public void setUp() {
    auditor = new ProtectConfigAuditor();
    auditor.setAuditRecorder(auditRecorder);
    when(auditRecorder.isEnabled()).thenReturn(true);
  }

  @Test
  public void testFirewallProtectionChange() {
    ProtectConfigChangedEvent event = new ProtectConfigChangedEvent(
        "protect.firewall", "protection-level-changed",
        "maven-proxy", "none", "quarantine");

    auditor.on(event);

    verify(auditRecorder).record(auditDataCaptor.capture());
    AuditData data = auditDataCaptor.getValue();
    assertThat(data.getDomain(), is("protect.firewall"));
    assertThat(data.getType(), is("protection-level-changed"));
    assertThat(data.getContext(), is("maven-proxy"));
    assertThat(data.getAttributes().get("from"), is("none"));
    assertThat(data.getAttributes().get("to"), is("quarantine"));
  }

  @Test
  public void testHealthCheckToggle() {
    ProtectConfigChangedEvent event = new ProtectConfigChangedEvent(
        "protect.healthcheck", "healthcheck-toggled",
        "npm-proxy", "disabled", "enabled");

    auditor.on(event);

    verify(auditRecorder).record(auditDataCaptor.capture());
    AuditData data = auditDataCaptor.getValue();
    assertThat(data.getDomain(), is("protect.healthcheck"));
    assertThat(data.getAttributes().get("from"), is("disabled"));
    assertThat(data.getAttributes().get("to"), is("enabled"));
  }
}
