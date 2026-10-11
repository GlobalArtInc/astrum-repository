package org.globalart.astrum.notifications.internal;

/**
 * The outcome of a test message.
 */
public class NotificationTestResultXO
{
  private boolean success;

  private String message;

  public NotificationTestResultXO() {
  }

  public NotificationTestResultXO(final boolean success, final String message) {
    this.success = success;
    this.message = message;
  }

  public boolean isSuccess() {
    return success;
  }

  public void setSuccess(final boolean success) {
    this.success = success;
  }

  public String getMessage() {
    return message;
  }

  public void setMessage(final String message) {
    this.message = message;
  }
}
