
package org.globalart.astrum.common.property;

import java.util.Optional;

import org.globalart.astrum.common.time.Time;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Helper to retrieve a system property typed value.
 */
public class SystemPropertiesHelper
{
  private static final Logger log = LoggerFactory.getLogger(SystemPropertiesHelper.class);

  private SystemPropertiesHelper() {
    // empty
  }

  public static int getInteger(final String key, final int defaultValue) {
    final String value = System.getProperty(key);

    if (value == null || value.trim().length() == 0) {
      return defaultValue;
    }

    try {
      return Integer.valueOf(value);
    }
    catch (NumberFormatException e) { // NOSONAR
      log.warn("Invalid integer '{}' for property '{}'. Defaulting to '{}'", value, key, defaultValue);
      return defaultValue;
    }
  }

  public static long getLong(final String key, final long defaultValue) {
    final String value = System.getProperty(key);

    if (value == null || value.trim().length() == 0) {
      return defaultValue;
    }

    try {
      return Long.valueOf(value);
    }
    catch (NumberFormatException e) { // NOSONAR
      log.warn("Invalid long '{}' for property '{}'. Defaulting to '{}'", value, key, defaultValue);
      return defaultValue;
    }
  }

  public static boolean getBoolean(final String key, final boolean defaultValue) {
    return getBoolean(key)
        .orElse(defaultValue);
  }

  public static Optional<Boolean> getBoolean(final String key) {
    return Optional.ofNullable(System.getProperty(key))
        .map(String::trim)
        .filter(value -> value.length() > 0)
        .map(Boolean::valueOf);
  }

  public static String getString(final String key, final String defaultValue) {
    return System.getProperty(key, defaultValue);
  }

  /**
   * @since 3.16
   */
  public static Time getTime(final String key, final Time defaultValue) {
    final String value = System.getProperty(key);

    if (value == null || value.trim().length() == 0) {
      return defaultValue;
    }

    try {
      return Time.parse(value);
    }
    catch (RuntimeException e) { // NOSONAR
      log.warn("Invalid time '{}' for property '{}'. Defaulting to '{}'", value, key, defaultValue);
      return defaultValue;
    }
  }
}
