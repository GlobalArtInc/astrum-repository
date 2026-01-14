
package org.globalart.astrum.webhooks;

import java.util.List;

/**
 * Webhook service.
 *
 * @since 3.1
 */
public interface WebhookService
{
  /**
   * Returns all detected webhooks.
   */
  List<Webhook> getWebhooks();

  /**
   * Queue a {@link WebhookRequest} to be delivered later asynchronously.
   *
   * @see WebhookRequestSendEvent
   */
  void queue(final WebhookRequest request);

  /**
   * Send a {@link WebhookRequest} immediately.
   */
  void send(final WebhookRequest request) throws Exception;
}
