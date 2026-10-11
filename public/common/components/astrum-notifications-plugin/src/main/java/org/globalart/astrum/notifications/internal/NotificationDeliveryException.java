package org.globalart.astrum.notifications.internal;

/**
 * A chat did not accept a message, or the message did not get through to it. The message never contains the bot token,
 * the webhook URL or the proxy password.
 */
public class NotificationDeliveryException
    extends Exception
{
  private final int status;

  private final boolean retryable;

  private final long retryAfterMillis;

  public NotificationDeliveryException(
      final String message,
      final int status,
      final boolean retryable,
      final long retryAfterMillis)
  {
    super(message);
    this.status = status;
    this.retryable = retryable;
    this.retryAfterMillis = retryAfterMillis;
  }

  public static NotificationDeliveryException permanent(final String message) {
    return new NotificationDeliveryException(message, 0, false, 0);
  }

  /**
   * @return the HTTP status the chat answered with, or 0 when there was no answer
   */
  public int getStatus() {
    return status;
  }

  /**
   * @return whether sending the same message again later may succeed
   */
  public boolean isRetryable() {
    return retryable;
  }

  /**
   * @return how long the chat asked to wait before the next attempt, or 0 when it did not say
   */
  public long getRetryAfterMillis() {
    return retryAfterMillis;
  }
}
