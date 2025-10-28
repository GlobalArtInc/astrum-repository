
package org.globalart.astrum.common.log;

/**
 * Logger levels supported.
 *
 * @since 2.1
 */
public enum LoggerLevel
{
  TRACE,
  DEBUG,
  INFO,
  WARN,

  /**
   * @since 2.7
   */
  ERROR,

  /**
   * @since 2.7
   */
  OFF,

  /**
   * Level will be calculated as effective level.
   *
   * @since 2.7
   */
  DEFAULT
}
