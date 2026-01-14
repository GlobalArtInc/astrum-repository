

package org.globalart.astrum.webhooks;

import org.globalart.astrum.common.event.EventManager;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Event to queue up and send {@link WebhookRequest} asynchronously.
 *
 * Users may post this event directly to the {@link EventManager} or use {@link WebhookService#queue(WebhookRequest)}.
 *
 * @since 3.1
 */
public class WebhookRequestSendEvent
{
  private final WebhookRequest request;

  public WebhookRequestSendEvent(final WebhookRequest request) {
    this.request = checkNotNull(request);
  }

  public WebhookRequest getRequest() {
    return request;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "request=" + request +
        '}';
  }
}
