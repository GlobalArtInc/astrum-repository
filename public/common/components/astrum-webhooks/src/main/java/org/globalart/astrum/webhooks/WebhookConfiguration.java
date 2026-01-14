
package org.globalart.astrum.webhooks;

import java.net.URI;

import javax.annotation.Nullable;

/**
 * Webhook configuration for a single {@link WebhookSubscription}.
 *
 * @see Webhook#subscribe(WebhookConfiguration)
 * @since 3.1
 */
public interface WebhookConfiguration
{
  /**
   * URL of remote endpoint to receive HTTP POST requests.
   */
  URI getUrl();

  /**
   * Optional secret for signing webhook body.
   */
  @Nullable
  String getSecret();
}
