
package org.globalart.astrum.common.log;

import javax.annotation.Nullable;

/**
 * Emitted when a {@link LoggerLevel} has been changed.
 *
 * @since 3.1
 */
public class LoggerLevelChangedEvent
{
  private final String logger;

  @Nullable
  private final LoggerLevel level;

  public LoggerLevelChangedEvent(final String logger, @Nullable final LoggerLevel level) {
    this.logger = logger;
    this.level = level;
  }

  public String getLogger() {
    return logger;
  }

  /**
   * Level may be null if the logger level was unset.
   */
  @Nullable
  public LoggerLevel getLevel() {
    return level;
  }

  @Override
  public String toString() {
    return getClass().getSimpleName() + "{" +
        "logger='" + logger + '\'' +
        ", level=" + level +
        '}';
  }
}
