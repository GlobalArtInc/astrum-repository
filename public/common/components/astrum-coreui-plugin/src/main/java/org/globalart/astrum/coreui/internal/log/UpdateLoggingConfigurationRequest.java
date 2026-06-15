
package org.globalart.astrum.coreui.internal.log;

import org.globalart.astrum.common.log.LoggerLevel;

/**
 * @since 3.24
 */
public class UpdateLoggingConfigurationRequest
{
  private LoggerLevel level;

  public LoggerLevel getLevel() {
    return level;
  }

  public void setLevel(final LoggerLevel level) {
    this.level = level;
  }
}
