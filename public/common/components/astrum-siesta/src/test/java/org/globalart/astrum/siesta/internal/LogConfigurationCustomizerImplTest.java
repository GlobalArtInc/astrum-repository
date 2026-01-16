
package org.globalart.astrum.siesta.internal;

import org.globalart.astrum.common.log.LogConfigurationCustomizer.Configuration;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;

import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.globalart.astrum.common.log.LoggerLevel.DEFAULT;

class LogConfigurationCustomizerImplTest
{
  private LogConfigurationCustomizerImpl customizer;

  private Configuration configuration;

  @BeforeEach
  void setUp() {
    customizer = new LogConfigurationCustomizerImpl();
    configuration = mock(Configuration.class);
  }

  @Test
  void testCustomize() {
    customizer.customize(configuration);

    verify(configuration).setLoggerLevel("org.globalart.astrum.rest", DEFAULT);
    verify(configuration).setLoggerLevel("org.globalart.astrum.plugins.siesta", DEFAULT);
    verify(configuration).setLoggerLevel("org.jboss.resteasy", DEFAULT);
  }
}
