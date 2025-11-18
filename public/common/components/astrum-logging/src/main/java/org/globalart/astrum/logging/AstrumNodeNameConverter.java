
package org.globalart.astrum.logging;

import ch.qos.logback.classic.pattern.ClassicConverter;
import ch.qos.logback.classic.spi.ILoggingEvent;

/**
 * Mapped from {@link AstrumLayoutEncoder}
 *
 * @since 3.6.1
 */
public class AstrumNodeNameConverter
    extends ClassicConverter
{

  @Override
  public String convert(final ILoggingEvent iLoggingEvent) {
    return System.getProperty("astrum.clustered.nodeName", "");
  }
}
