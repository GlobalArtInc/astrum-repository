
package org.globalart.astrum.common.log;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Emitted when the log has been marked.
 *
 * @since 3.1
 */
public class LogMarkInsertedEvent
{
  private final String message;

  public LogMarkInsertedEvent(final String message) {
    this.message = checkNotNull(message);
  }

  public String getMessage() {
    return message;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{message='" + message + "'}";
  }
}
