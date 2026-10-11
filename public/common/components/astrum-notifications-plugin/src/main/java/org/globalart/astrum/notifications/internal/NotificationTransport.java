package org.globalart.astrum.notifications.internal;

/**
 * Sends one message to the chat of a channel.
 */
public interface NotificationTransport
{
  /**
   * @param channel the channel to send to
   * @param secrets the channel's decrypted secrets
   * @param text the message, formatted for the channel's chat
   */
  void send(NotificationChannel channel, NotificationSecrets secrets, String text) throws NotificationDeliveryException;
}
