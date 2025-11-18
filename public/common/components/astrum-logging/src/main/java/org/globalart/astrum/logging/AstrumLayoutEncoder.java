
package org.globalart.astrum.logging;

import ch.qos.logback.classic.PatternLayout;
import ch.qos.logback.classic.spi.ILoggingEvent;
import ch.qos.logback.core.pattern.PatternLayoutEncoderBase;

/**
 * Adds the ability to use %node in an encoder pattern
 *
 * @since 3.6.1
 */
public class AstrumLayoutEncoder
    extends PatternLayoutEncoderBase<ILoggingEvent>
{
  @Override
  public void start() {
    PatternLayout patternLayout = new PatternLayout();
    patternLayout.getDefaultConverterMap().put("node", AstrumNodeNameConverter.class.getName());
    patternLayout.setContext(context);
    patternLayout.setPattern(getPattern());
    patternLayout.start();
    this.layout = patternLayout;
    super.start();
  }
}
