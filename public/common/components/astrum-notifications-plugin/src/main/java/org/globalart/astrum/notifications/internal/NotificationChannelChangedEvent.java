package org.globalart.astrum.notifications.internal;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Posted when an administrator creates, changes or deletes a notification channel.
 */
public class NotificationChannelChangedEvent
{
  public enum Type
  {
    CREATED,
    UPDATED,
    DELETED
  }

  private final Type type;

  private final NotificationChannel channel;

  public NotificationChannelChangedEvent(final Type type, final NotificationChannel channel) {
    this.type = checkNotNull(type);
    this.channel = checkNotNull(channel).copy();
  }

  public Type getType() {
    return type;
  }

  public NotificationChannel getChannel() {
    return channel;
  }
}
