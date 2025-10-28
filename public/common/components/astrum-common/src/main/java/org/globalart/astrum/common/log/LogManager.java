
package org.globalart.astrum.common.log;

import java.io.File;
import java.io.IOException;
import java.io.InputStream;
import java.nio.file.Path;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import javax.annotation.Nullable;

import org.globalart.astrum.common.lifecycle.Lifecycle;

/**
 * Log manager.
 */
public interface LogManager
    extends Lifecycle
{
  String DEFAULT_LOGGER = "logfile";

  static final String TASKS_PREFIX = "tasks/";

  static final String REPLICATION_PREFIX = "replication/";

  Set<File> getLogFiles();

  @Nullable
  File getLogFile(String fileName);

  boolean isValidLogFile(Path filepath);

  /**
   * If the named logger is a file based log, return the file name associated with it.
   *
   * @since 3.17
   * @param loggerName name of the logger whose log file's name, if applicable, should be returned
   * @return The file name to which the named logger appends
   */
  Optional<String> getLogFor(String loggerName);

  /**
   * If the named logger is a file based log, return the file associated with it.
   *
   * @since 3.17
   * @param loggerName name of the logger whose log file, if applicable, should be returned
   * @return The file to which the named logger appends
   */
  Optional<File> getLogFileForLogger(String loggerName);

  /**
   * Provides access to named log-file streams.
   *
   * @param fileName name of log file to fetch
   * @return Stream to log file or {@code null} if non-existent.
   */
  @Nullable
  InputStream getLogFileStream(String fileName, long fromByte, long bytesCount) throws IOException;

  /**
   * Provides access to named log-file streams.
   *
   * @param fileName name of log file to fetch
   * @return Stream to log file or {@code null} if non-existent.
   */
  @Nullable
  InputStream getLogFileStream(String fileName) throws IOException;

  /**
   * Return mapping of existing loggers which have explicit levels configured (never null).
   *
   * @since 2.7
   */
  Map<String, LoggerLevel> getLoggers();

  /**
   * Return mapping of existing loggers which have been overridden by the user.
   *
   * @since 3.2
   */
  Map<String, LoggerLevel> getOverriddenLoggers();

  /**
   * @since 2.7
   */
  void resetLoggers();

  /**
   * @since 2.7
   */
  void setLoggerLevel(String name, @Nullable LoggerLevel level);

  void setLoggerLevelDirect(String name, @Nullable LoggerLevel level);

  /**
   * @since 2.7
   */
  void unsetLoggerLevel(String name);

  /**
   * @since 2.7
   */
  @Nullable
  LoggerLevel getLoggerLevel(String name);

  /**
   * @since 2.7
   */
  LoggerLevel getLoggerEffectiveLevel(String name);

  /**
   * Return effective loggers map(logger name - log level), updated by latest overrides fetched from datastore
   * that might not be yet propagated by DES
   */
  Map<String, LoggerLevel> getEffectiveLoggersUpdatedByFetchedOverrides();

  /**
   * @return true if GlobalArt's custom RemoteTimeBasedRollingPolicy is enabled
   */
  boolean isRemoteTimeBasedRollingPolicyEnabled();
}
