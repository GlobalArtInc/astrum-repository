package org.globalart.astrum.notifications.internal;

/**
 * Sends one message to the chat or service of a channel.
 */
public interface NotificationTransport
{
  /**
   * @param channel the channel to send to
   * @param secrets the channel's decrypted secrets
   * @param message the message, its text formatted for the channel's chat
   */
  void send(NotificationChannel channel, NotificationSecrets secrets, NotificationMessage message)
      throws NotificationDeliveryException;
}
