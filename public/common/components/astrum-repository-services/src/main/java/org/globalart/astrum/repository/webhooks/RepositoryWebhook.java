
package org.globalart.astrum.repository.webhooks;

import org.globalart.astrum.webhooks.Webhook;
import org.globalart.astrum.webhooks.WebhookConfiguration;
import org.globalart.astrum.webhooks.WebhookType;

/**
 * Repository {@link Webhook}.
 *
 * @since 3.1
 */
public abstract class RepositoryWebhook
    extends Webhook
{
  public static final WebhookType TYPE = new WebhookType("repository")
  {
  };

  public final WebhookType getType() {
    return TYPE;
  }

  /**
   * Additional configuration exposed for {@link RepositoryWebhook}.
   */
  public interface Configuration
      extends WebhookConfiguration
  {
    String getRepository();
  }
}
