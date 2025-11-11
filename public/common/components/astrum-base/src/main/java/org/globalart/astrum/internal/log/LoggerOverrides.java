
package org.globalart.astrum.internal.log;

import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.common.log.LoggerLevel;

/**
 * Manages user-configured logger overrides.
 *
 * @since 3.0
 */
public interface LoggerOverrides
    extends Iterable<Map.Entry<String, LoggerLevel>>
{
  /**
   * Load overrides configuration from storage.
   */
  void load();

  /**
   * Save current overrides configuration to storage.
   */
  void save();

  /**
   * Update local logger overrides using datastore and return 'logger name' to 'log level' map
   */
  Map<String, LoggerLevel> syncWithDBAndGet();

  /**
   * Reset overrides configuration to default state.
   */
  void reset();

  /**
   * Set a logger level override.
   */
  void set(String name, LoggerLevel level);

  /**
   * Get the level of an overridden logger.
   */
  @Nullable
  LoggerLevel get(final String name);

  /**
   * Remove override configuration for a logger.
   */
  @Nullable
  LoggerLevel remove(final String name);

  /**
   * Check if a logger has been overridden.
   */
  boolean contains(final String name);
}
