
package org.globalart.astrum.webhooks;

/**
 * Global {@link Webhook}.
 *
 * @since 3.1
 */
public abstract class GlobalWebhook
    extends Webhook
{
  public static final WebhookType TYPE = new WebhookType("global")
  {
  };

  public final WebhookType getType() {
    return TYPE;
  }
}
