package org.globalart.astrum.notifications.internal;

import java.util.Arrays;
import java.util.Optional;

/**
 * What happened to a component, as a channel subscribes to it.
 */
public enum NoticeAction
{
  PUBLISHED("published"),
  UPDATED("updated"),
  DELETED("deleted");

  private final String id;

  NoticeAction(final String id) {
    this.id = id;
  }

  public String id() {
    return id;
  }

  public static Optional<NoticeAction> of(final String id) {
    return Arrays.stream(values()).filter(action -> action.id.equals(id)).findFirst();
  }
}
