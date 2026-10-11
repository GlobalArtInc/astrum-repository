package org.globalart.astrum.notifications.internal;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * The decrypted secrets a channel needs to send one message. They never appear in logs or messages.
 *
 * @param secret the bot token or the webhook URL
 * @param proxyPassword the password of the channel's own proxy, or {@code null} when it has none
 * @param signingSecret the key JSON webhook bodies are signed with, or {@code null} when they are not signed
 * @param authorization the Authorization header of JSON webhook requests, or {@code null} when they have none
 */
public record NotificationSecrets(
    String secret,
    @Nullable String proxyPassword,
    @Nullable String signingSecret,
    @Nullable String authorization)
{
  public NotificationSecrets {
    checkNotNull(secret);
  }

  public static NotificationSecrets of(final String secret) {
    return new NotificationSecrets(secret, null, null, null);
  }

  @Override
  public String toString() {
    return "NotificationSecrets{***}";
  }
}
