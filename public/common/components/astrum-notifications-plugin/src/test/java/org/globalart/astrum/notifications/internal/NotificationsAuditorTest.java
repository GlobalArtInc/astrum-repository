package org.globalart.astrum.notifications.internal;

import java.util.List;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditRecorder;
import org.globalart.astrum.notifications.internal.NotificationChannelChangedEvent.Type;

import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.hasKey;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

public class NotificationsAuditorTest
{
  private AuditRecorder recorder;

  private NotificationsAuditor auditor;

  @Before
  public void setUp() {
    recorder = mock(AuditRecorder.class);
    when(recorder.isEnabled()).thenReturn(true);
    auditor = new NotificationsAuditor();
    auditor.setAuditRecorder(recorder);
  }

  private AuditData recorded(final NotificationChannel channel) {
    auditor.on(new NotificationChannelChangedEvent(Type.UPDATED, channel));
    ArgumentCaptor<AuditData> data = ArgumentCaptor.forClass(AuditData.class);
    verify(recorder).record(data.capture());
    return data.getValue();
  }

  @Test
  public void recordsTheProxyOfAChannelButNeverItsSecrets() {
    NotificationChannel channel = new NotificationChannel();
    channel.setId("c1");
    channel.setName("releases");
    channel.setType(NotificationChannel.TELEGRAM);
    channel.setChatId("-100");
    channel.setSecretId("_token");
    channel.setEvents(List.of("published"));
    channel.setProxyMode(NotificationChannel.PROXY_CUSTOM);
    channel.setProxyType(NotificationChannel.PROXY_SOCKS5);
    channel.setProxyHost("proxy.internal");
    channel.setProxyPort(1080);
    channel.setProxyUsername("robot");
    channel.setProxyPasswordSecretId("_password");

    AuditData data = recorded(channel);

    assertThat(data.getType(), is(NotificationsAuditor.UPDATED_TYPE));
    assertThat(data.getAttributes(), hasEntry("proxyMode", "custom"));
    assertThat(data.getAttributes(), hasEntry("proxyType", "socks5"));
    assertThat(data.getAttributes(), hasEntry("proxyHost", "proxy.internal"));
    assertThat(data.getAttributes(), hasEntry("proxyPort", "1080"));
    assertThat(data.getAttributes().toString().contains("_password"), is(false));
    assertThat(data.getAttributes().toString().contains("_token"), is(false));
  }

  @Test
  public void recordsOnlyTheModeOfOtherConnections() {
    NotificationChannel channel = new NotificationChannel();
    channel.setId("c2");
    channel.setName("builds");
    channel.setType(NotificationChannel.SLACK);
    channel.setTargetHost("hooks.slack.com");
    channel.setProxyMode(NotificationChannel.PROXY_NONE);

    AuditData data = recorded(channel);

    assertThat(data.getAttributes(), hasEntry("proxyMode", "none"));
    assertThat(data.getAttributes(), not(hasKey("proxyHost")));
  }
}
