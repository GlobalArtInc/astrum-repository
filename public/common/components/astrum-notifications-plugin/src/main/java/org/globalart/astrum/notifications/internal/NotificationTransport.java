package org.globalart.astrum.notifications.internal;

/**
 * Sends one message to the chat of a channel.
 */
public interface NotificationTransport
{
  /**
   * @param channel the channel to send to
   * @param secret the channel's bot token or webhook URL
   * @param text the message, formatted for the channel's chat
   */
  void send(NotificationChannel channel, String secret, String text) throws NotificationDeliveryException;
}
