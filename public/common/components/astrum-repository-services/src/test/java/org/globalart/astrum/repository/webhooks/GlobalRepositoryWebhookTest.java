
package org.globalart.astrum.repository.webhooks;

import org.globalart.astrum.audit.InitiatorProvider;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryEvent;
import org.globalart.astrum.repository.manager.RepositoryCreatedEvent;
import org.globalart.astrum.repository.manager.RepositoryDeletedEvent;
import org.globalart.astrum.repository.manager.RepositoryUpdatedEvent;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.webhooks.GlobalRepositoryWebhook.RepositoryWebhookPayload;
import org.globalart.astrum.webhooks.WebhookRequestSendEvent;

import com.google.common.collect.Lists;
import org.junit.Before;
import org.junit.Test;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.core.Is.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.hamcrest.Matchers.equalTo;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class GlobalRepositoryWebhookTest
{
  @Mock
  private EventManager eventManager;

  @Mock
  private InitiatorProvider initiatorProvider;

  @Mock
  private NodeAccess nodeAccess;

  @Mock
  private RepositoryCreatedEvent repositoryCreatedEvent;

  @Mock
  private RepositoryUpdatedEvent repositoryUpdatedEvent;

  @Mock
  private RepositoryDeletedEvent repositoryDeletedEvent;

  private GlobalRepositoryWebhook globalRepositoryWebhook;

  @Before
  public void before() {
    globalRepositoryWebhook = new GlobalRepositoryWebhook(nodeAccess, initiatorProvider);
    globalRepositoryWebhook.setEventManager(eventManager);

    for (RepositoryEvent repositoryEvent : Lists
        .newArrayList(repositoryCreatedEvent, repositoryDeletedEvent, repositoryUpdatedEvent)) {
      Repository repository = mock(Repository.class);
      when(repository.getName()).thenReturn("name");
      when(repository.getFormat()).thenReturn(new TestFormat());
      when(repository.getType()).thenReturn(new ProxyType());

      when(repositoryEvent.getRepository()).thenReturn(repository);
    }

    RepositoryWebhook.Configuration configuration = mock(RepositoryWebhook.Configuration.class);
    when(configuration.getRepository()).thenReturn("repoName");
    globalRepositoryWebhook.subscribe(configuration);

    when(initiatorProvider.get()).thenReturn("initiator");
    when(nodeAccess.getId()).thenReturn("nodeId");
  }

  @Test
  public void hasTheCorrectEventId() {
    assertThat(globalRepositoryWebhook.getId(), is(equalTo("rm:global:repository")));
  }

  @Test
  public void queuesRepositoryCreatedEvents() {
    globalRepositoryWebhook.on(repositoryCreatedEvent);
    testRepositoryEvent("CREATED");
  }

  @Test
  public void queuesRepositoryUpdatedEvents() {
    globalRepositoryWebhook.on(repositoryUpdatedEvent);
    testRepositoryEvent("UPDATED");
  }

  @Test
  public void queuesRepositoryDeletedEvents() {
    globalRepositoryWebhook.on(repositoryDeletedEvent);
    testRepositoryEvent("DELETED");
  }

  private void testRepositoryEvent(String action) {
    ArgumentCaptor<WebhookRequestSendEvent> assetArgumentCaptor =
        ArgumentCaptor.forClass(WebhookRequestSendEvent.class);
    verify(eventManager).post(assetArgumentCaptor.capture());

    RepositoryWebhookPayload repositoryPayload =
        (RepositoryWebhookPayload) assetArgumentCaptor.getValue().getRequest().getPayload();

    assertThat(repositoryPayload.getInitiator(), is(equalTo("initiator")));
    assertThat(repositoryPayload.getNodeId(), is(equalTo("nodeId")));
    assertThat(repositoryPayload.getAction().toString(), is(equalTo(action)));

    assertThat(repositoryPayload.getRepository().getName(), is(equalTo("name")));
    assertThat(repositoryPayload.getRepository().getFormat(), is(equalTo("format")));
    assertThat(repositoryPayload.getRepository().getType(), is(equalTo("proxy")));
  }

  public class TestFormat
      extends Format
  {
    public TestFormat() {
      super("format");
    }
  }
}
