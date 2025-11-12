
package org.globalart.astrum.internal.log.overrides.file;

import java.io.File;
import java.util.HashMap;
import java.util.Iterator;
import java.util.Map;
import java.util.Map.Entry;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.bootstrap.entrypoint.configuration.ApplicationDirectories;
import org.globalart.astrum.common.log.LoggerLevel;
import org.globalart.astrum.internal.log.overrides.LogbackLoggerOverridesSupport;
import org.globalart.astrum.internal.log.LoggerOverrides;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.base.Throwables;
import com.google.common.collect.ImmutableMap;
import org.springframework.stereotype.Component;

/**
 * Logback {@link LoggerOverrides} implementation.
 *
 * Special handling for {@code ROOT} logger, which is persisted as {@code root.level} property.
 *
 * @since 2.7
 */
@Component
public class LogbackLoggerOverrides
    extends LogbackLoggerOverridesSupport
    implements LoggerOverrides
{
  private final Map<String, LoggerLevel> loggerLevels = new HashMap<>();

  @Autowired
  public LogbackLoggerOverrides(final ApplicationDirectories applicationDirectories) {
    super(applicationDirectories);
  }

  @VisibleForTesting
  LogbackLoggerOverrides(final File file) {
    super(file);
  }

  @Override
  public synchronized void load() {
    log.debug("Load");

    loggerLevels.clear();
    if (logbackFileExists()) {
      try {
        loggerLevels.putAll(readFromFile());
      }
      catch (Exception e) {
        Throwables.throwIfUnchecked(e);
        throw new RuntimeException(e);
      }
    }
  }

  @Override
  public synchronized void save() {
    log.debug("Save");

    try {
      writeToFile(loggerLevels);
    }
    catch (Exception e) {
      Throwables.throwIfUnchecked(e);
      throw new RuntimeException(e);
    }
  }

  @Override
  public Map<String, LoggerLevel> syncWithDBAndGet() {
    // Not applicable to Orient
    return loggerLevels;
  }

  @Override
  public synchronized void reset() {
    log.debug("Reset");

    loggerLevels.clear();
    try {
      writeToFile(loggerLevels);
    }
    catch (Exception e) {
      Throwables.throwIfUnchecked(e);
      throw new RuntimeException(e);
    }
  }

  @Override
  public synchronized void set(final String name, final LoggerLevel level) {
    log.debug("Set: {}={}", name, level);

    loggerLevels.put(name, level);
  }

  @Override
  @Nullable
  public synchronized LoggerLevel get(final String name) {
    return loggerLevels.get(name);
  }

  @Override
  @Nullable
  public synchronized LoggerLevel remove(final String name) {
    log.debug("Remove: {}", name);

    return loggerLevels.remove(name);
  }

  @Override
  public synchronized boolean contains(final String name) {
    return loggerLevels.containsKey(name);
  }

  @Override
  public synchronized Iterator<Entry<String, LoggerLevel>> iterator() {
    return ImmutableMap.copyOf(loggerLevels).entrySet().iterator();
  }
}
