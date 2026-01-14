
package org.globalart.astrum.webhooks;

import java.net.URI;
import java.util.UUID;

import javax.annotation.Nullable;

/**
 * Webhook request.
 *
 * @since 3.1
 */
public class WebhookRequest
{
  private final String id = UUID.randomUUID().toString();

  private Webhook webhook;

  private WebhookPayload payload;

  private URI url;

  @Nullable
  private String secret;

  public String getId() {
    return id;
  }

  public Webhook getWebhook() {
    return webhook;
  }

  public void setWebhook(final Webhook webhook) {
    this.webhook = webhook;
  }

  public WebhookPayload getPayload() {
    return payload;
  }

  public void setPayload(final WebhookPayload payload) {
    this.payload = payload;
  }

  public URI getUrl() {
    return url;
  }

  public void setUrl(final URI url) {
    this.url = url;
  }

  @Nullable
  public String getSecret() {
    return secret;
  }

  public void setSecret(@Nullable final String secret) {
    this.secret = secret;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "id='" + id + '\'' +
        ", webhook=" + webhook +
        ", payload=" + payload +
        ", url=" + url +
        '}';
  }
}
