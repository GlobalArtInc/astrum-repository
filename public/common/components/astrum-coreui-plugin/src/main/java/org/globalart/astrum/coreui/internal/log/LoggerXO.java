
package org.globalart.astrum.coreui.internal.log;

import java.util.Map;
import jakarta.validation.constraints.NotEmpty;
import jakarta.validation.constraints.NotNull;

import org.globalart.astrum.common.log.LoggerLevel;

/**
 * Logger exchange object.
 */
public class LoggerXO
{
  @NotEmpty
  private String name;

  @NotNull
  private LoggerLevel level;

  private boolean override;

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public LoggerLevel getLevel() {
    return level;
  }

  public void setLevel(final LoggerLevel level) {
    this.level = level;
  }

  public boolean isOverride() {
    return override;
  }

  public void setOverride(final boolean override) {
    this.override = override;
  }

  @Override
  public String toString() {
    return "LoggerXO(" +
        "name:" + name +
        ", level:" + level +
        ", override:" + override +
        ")";
  }

  public static LoggerXO fromEntry(Map.Entry<String, LoggerLevel> entry) {
    LoggerXO loggerXO = new LoggerXO();
    loggerXO.setName(entry.getKey());
    loggerXO.setLevel(entry.getValue());
    return loggerXO;
  }
}
