
package org.globalart.astrum.webhooks;

/**
 * Webhook subscription.
 *
 * @see Webhook#subscribe(WebhookConfiguration)
 * @since 3.1
 */
public interface WebhookSubscription
{
  /**
   * Returns the configuration for this subscription.
   */
  WebhookConfiguration getConfiguration();

  /**
   * Cancel this subscription.
   *
   * @see Webhook#unsubscribe(WebhookSubscription)
   */
  void cancel();
}
