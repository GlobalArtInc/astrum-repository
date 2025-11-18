
package org.globalart.astrum.logging.task;

import org.slf4j.Logger;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Arrays.copyOf;

/**
 * Simple object to hold a log message and args, so it can be logged as progress later
 *
 * @since 3.5
 */
public class TaskLoggingEvent
{
  private final Logger logger;

  private final String message;

  private final Object[] args;

  public TaskLoggingEvent(final Logger logger, final String message) {
    this(logger, message, null);
  }

  public TaskLoggingEvent(final Logger logger, final String message, final Object[] args) {
    this.logger = checkNotNull(logger);
    this.message = checkNotNull(message);
    this.args = args == null ? null : copyOf(args, args.length);
  }

  public Logger getLogger() {
    return logger;
  }

  public String getMessage() {
    return message;
  }

  public Object[] getArgumentArray() {
    return args;
  }
}
