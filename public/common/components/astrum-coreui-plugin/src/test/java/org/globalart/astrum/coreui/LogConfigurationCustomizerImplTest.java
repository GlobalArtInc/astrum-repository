
package org.globalart.astrum.coreui;

import org.globalart.astrum.common.log.LogConfigurationCustomizer.Configuration;
import org.globalart.astrum.common.log.LoggerLevel;

import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.Mock;

import static org.mockito.Mockito.verify;

@ExtendWith(MockitoExtension.class)
class LogConfigurationCustomizerImplTest
{
  @Mock
  Configuration configuration;

  @Test
  void customizeSetsCoreuiLoggerToDefaultLevel() {
    LogConfigurationCustomizerImpl underTest = new LogConfigurationCustomizerImpl();

    underTest.customize(configuration);

    verify(configuration).setLoggerLevel("org.globalart.astrum.coreui", LoggerLevel.DEFAULT);
  }
}
