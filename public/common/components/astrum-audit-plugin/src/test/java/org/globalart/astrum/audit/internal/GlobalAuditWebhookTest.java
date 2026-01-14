
package org.globalart.astrum.audit.internal;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.audit.AuditData;
import org.globalart.astrum.audit.AuditDataRecordedEvent;
import org.globalart.astrum.audit.internal.GlobalAuditWebhook.AuditWebhookPayload;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.webhooks.WebhookConfiguration;
import org.globalart.astrum.webhooks.WebhookRequestSendEvent;

import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class GlobalAuditWebhookTest
{

  @Mock
  private EventManager eventManager;

  private GlobalAuditWebhook underTest;

  @Captor
  private ArgumentCaptor<WebhookRequestSendEvent> argumentCaptor;

  @Before
  public void setUp() {
    underTest = new GlobalAuditWebhook();
    underTest.setEventManager(eventManager);
  }

  @Test
  public void testShouldHaveTheCorrectEventId() {
    assertThat(underTest.getId(), is("rm:global:audit"));
  }

  @Test
  public void testShouldQueueAuditWebhook() {
    WebhookConfiguration configuration = mock(WebhookConfiguration.class);
    underTest.subscribe(configuration);

    AuditData auditData = new AuditData();
    auditData.setAttributes(new HashMap<>(Map.of("key", "value")));
    auditData.setInitiator("initiator");
    auditData.setNodeId("nodeId");
    auditData.setDomain("domain");
    auditData.setContext("context");
    auditData.setType("type");

    AuditDataRecordedEvent auditRecordedEvent = new AuditDataRecordedEvent(auditData);

    underTest.on(auditRecordedEvent);

    verify(eventManager).post(argumentCaptor.capture());

    AuditWebhookPayload auditPayload = (AuditWebhookPayload) argumentCaptor.getValue().getRequest().getPayload();

    assertThat(auditPayload.getInitiator(), is("initiator"));
    assertThat(auditPayload.getNodeId(), is("nodeId"));
    assertThat(auditPayload.getAudit().getDomain(), is("domain"));
    assertThat(auditPayload.getAudit().getType(), is("type"));
    assertThat(auditPayload.getAudit().getContext(), is("context"));
    assertThat(auditPayload.getAudit().getAttributes().size(), is(1));
    assertThat(auditPayload.getAudit().getAttributes(), hasEntry("key", "value"));
  }
}
