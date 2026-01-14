
package org.globalart.astrum.webhooks;

import static com.google.common.base.Preconditions.checkNotNull;

// NOTE: this is probably better named as WebhookScope

/**
 * Webhook type symbol.
 *
 * @since 3.1
 */
public abstract class WebhookType
{
  private final String value;

  protected WebhookType(final String value) {
    this.value = checkNotNull(value);
  }

  public String getValue() {
    return value;
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof WebhookType)) {
      return false;
    }

    WebhookType that = (WebhookType) o;
    return value.equals(that.value);
  }

  @Override
  public int hashCode() {
    return value.hashCode();
  }

  @Override
  public String toString() {
    return value;
  }
}
