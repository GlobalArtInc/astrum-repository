package org.globalart.astrum.notifications.internal;

import java.time.Instant;
import java.util.UUID;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * One message for a channel: the text a chat shows and, for JSON webhooks, the event and the change it tells about.
 * Its delivery id and time are fixed when it is written, so that a message sent again is recognisably the same.
 *
 * @param event {@code component.published}, {@code component.updated}, {@code component.deleted}, {@code test}, or
 *          {@code component.summary} for a summary of many changes, which JSON webhooks never get
 * @param text the message as the channel's chat shows it
 * @param notice the change the message tells about, if it is about one
 * @param deliveryId identifies the message
 * @param timestamp when the message was written
 */
public record NotificationMessage(
    String event,
    String text,
    @Nullable ComponentNotice notice,
    String deliveryId,
    Instant timestamp)
{
  public static final String TEST = "test";

  public static final String SUMMARY = "component.summary";

  public NotificationMessage {
    checkNotNull(event);
    checkNotNull(text);
    checkNotNull(deliveryId);
    checkNotNull(timestamp);
  }

  public static NotificationMessage of(final ComponentNotice notice, final String text) {
    return new NotificationMessage("component." + notice.action().id(), text, notice, UUID.randomUUID().toString(),
        Instant.now());
  }

  public static NotificationMessage test(final String text) {
    return new NotificationMessage(TEST, text, null, UUID.randomUUID().toString(), Instant.now());
  }

  public static NotificationMessage summary(final String text) {
    return new NotificationMessage(SUMMARY, text, null, UUID.randomUUID().toString(), Instant.now());
  }
}
