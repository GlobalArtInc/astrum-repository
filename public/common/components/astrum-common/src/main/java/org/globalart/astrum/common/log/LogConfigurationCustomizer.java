
package org.globalart.astrum.common.log;

/**
 * Allows participation to logging configuration.
 *
 * @since 2.7
 */
public interface LogConfigurationCustomizer
{
  /**
   * Callback allowing setting of loggers levels.
   */
  void customize(Configuration configuration);

  interface Configuration
  {
    /**
     * Sets logger level to specified value.
     *
     * If logger level is {@link LoggerLevel#DEFAULT} level will be calculated as effective level.
     *
     * @param name logger name
     * @param level logger level
     */
    void setLoggerLevel(String name, LoggerLevel level);
  }
}
