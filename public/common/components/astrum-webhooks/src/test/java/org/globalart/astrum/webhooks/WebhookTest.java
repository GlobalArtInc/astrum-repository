
package org.globalart.astrum.webhooks;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import org.globalart.astrum.common.event.EventManager;

import java.util.Set;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.*;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class WebhookTest
{
  @Mock
  private EventManager eventManager;

  private static class TestWebhookType
      extends WebhookType
  {
    TestWebhookType() {
      super("test");
    }
  }

  private static class TestWebhook
      extends Webhook
  {
    @Override
    public WebhookType getType() {
      return new TestWebhookType();
    }

    @Override
    public String getName() {
      return "test";
    }

    @Override
    public Set<WebhookSubscription> getSubscriptions() {
      return super.getSubscriptions();
    }
  }

  private TestWebhook underTest;

  @Before
  public void setUp() {
    underTest = new TestWebhook();
    underTest.setEventManager(eventManager);
  }

  @Test
  public void prependsRmToEventId() {
    assertThat(underTest.getId().startsWith("rm:"), is(true));
  }

  @Test
  public void listenForEventsOnFirstSubscription() {
    assertThat(underTest.getSubscriptions(), empty());

    underTest.subscribe(mock(WebhookConfiguration.class));
    assertThat(underTest.getSubscriptions().size(), is(1));

    underTest.subscribe(mock(WebhookConfiguration.class));
    assertThat(underTest.getSubscriptions().size(), is(2));

    verify(eventManager).register(underTest);
  }

  @Test
  public void stopListeningForEventsWhenSubscriptionsEmpty() {
    WebhookSubscription subscription1 = underTest.subscribe(mock(WebhookConfiguration.class));
    WebhookSubscription subscription2 = underTest.subscribe(mock(WebhookConfiguration.class));

    subscription1.cancel();
    assertThat(underTest.getSubscriptions().size(), is(1));

    subscription2.cancel();
    assertThat(underTest.getSubscriptions(), empty());

    verify(eventManager).unregister(underTest);
  }
}
