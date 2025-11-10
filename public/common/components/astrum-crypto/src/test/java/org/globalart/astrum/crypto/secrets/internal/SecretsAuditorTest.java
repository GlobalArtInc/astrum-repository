
package org.globalart.astrum.crypto.secrets.internal;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.common.event.EventHelper;
import org.globalart.astrum.crypto.secrets.ActiveKeyChangeEvent;

import com.google.common.collect.ImmutableMap;
import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;
import org.mockito.Spy;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SecretsAuditorTest
{
  @Mock
  private AuditRecorder auditRecorder;

  @Spy
  private SecretsAuditor underTest = new SecretsAuditor();

  @Captor
  private ArgumentCaptor<AuditData> captor;

  @Before
  public void setup() {
    underTest.setAuditRecorder(auditRecorder);
  }

  @Test
  public void audit() {
    when(auditRecorder.isEnabled()).thenReturn(true);

    underTest.on(new ActiveKeyChangeEvent("foo", "bar", "baz"));

    verify(auditRecorder, times(1)).record(captor.capture());

    AuditData data = captor.getValue();

    assertThat(data, notNullValue());
    assertThat(data.getContext(), is("system"));
    assertThat(data.getDomain(), is(SecretsAuditor.DOMAIN));
    assertThat(data.getType(), is("changed"));
    assertThat(data.getAttributes(), is(ImmutableMap.of("newKeyId", "foo", "previousKeyId", "bar", "userId", "baz")));
  }

  /*
   * Verifies that if the event originated on another node then the event is not recorded
   */
  @Test
  public void audit_replicatingEventIgnored() {
    when(auditRecorder.isEnabled()).thenReturn(true);

    EventHelper.asReplicating(() -> underTest.on(new ActiveKeyChangeEvent("foo", "bar", "baz")));

    verify(auditRecorder, never()).record(any());
  }

  /*
   * Verifies that when auditing is disabled then the event is not recorded
   */
  @Test
  public void audit_disabled() {
    when(auditRecorder.isEnabled()).thenReturn(false);

    underTest.on(new ActiveKeyChangeEvent("foo", "bar", "baz"));

    verify(auditRecorder, never()).record(any());
  }
}
