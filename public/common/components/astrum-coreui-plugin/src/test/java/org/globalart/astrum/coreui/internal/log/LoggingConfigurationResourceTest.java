
package org.globalart.astrum.coreui.internal.log;

import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.Map;

import org.globalart.astrum.common.log.LogManager;
import org.globalart.astrum.common.log.LoggerLevel;
import org.globalart.astrum.repository.BadRequestException;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension;
import org.globalart.astrum.testcommon.extensions.AuthenticationExtension.WithUser;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;
import static org.mockito.Mockito.doThrow;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
@ExtendWith(AuthenticationExtension.class)
@WithUser
class LoggingConfigurationResourceTest
{
  @Mock
  private LogManager logManager;

  private LoggingConfigurationResource underTest;

  @BeforeEach
  void setup() {
    underTest = new LoggingConfigurationResource(logManager);
  }

  @Test
  void readAll_returnsLoggers() {
    Map<String, LoggerLevel> loggers = new LinkedHashMap<>();
    loggers.put("ROOT", LoggerLevel.INFO);
    loggers.put("org.globalart.astrum", LoggerLevel.DEBUG);
    when(logManager.getEffectiveLoggersUpdatedByFetchedOverrides()).thenReturn(loggers);

    Collection<LoggerXO> result = underTest.readAll();

    assertThat(result, is(notNullValue()));
    assertThat(result, hasSize(2));
  }

  @Test
  void readAll_emptyLoggers() {
    when(logManager.getEffectiveLoggersUpdatedByFetchedOverrides()).thenReturn(Map.of());

    Collection<LoggerXO> result = underTest.readAll();

    assertThat(result, hasSize(0));
  }

  @Test
  void read_returnsLoggerWithOverride() {
    when(logManager.getLoggerEffectiveLevel("org.globalart.astrum")).thenReturn(LoggerLevel.DEBUG);
    Map<String, LoggerLevel> loggers = Map.of("org.globalart.astrum", LoggerLevel.DEBUG);
    when(logManager.getLoggers()).thenReturn(loggers);

    LoggerXO result = underTest.read("org.globalart.astrum");

    assertThat(result, is(notNullValue()));
    assertThat(result.getName(), is("org.globalart.astrum"));
    assertThat(result.getLevel(), is(LoggerLevel.DEBUG));
    assertThat(result.isOverride(), is(true));
  }

  @Test
  void read_returnsLoggerWithoutOverride() {
    when(logManager.getLoggerEffectiveLevel("org.globalart.astrum")).thenReturn(LoggerLevel.INFO);
    when(logManager.getLoggers()).thenReturn(Map.of());

    LoggerXO result = underTest.read("org.globalart.astrum");

    assertThat(result.getName(), is("org.globalart.astrum"));
    assertThat(result.getLevel(), is(LoggerLevel.INFO));
    assertThat(result.isOverride(), is(false));
  }

  @Test
  void resetAll_delegatesToLogManager() {
    underTest.resetAll();

    verify(logManager).resetLoggers();
  }

  @Test
  void update_setsLoggerLevel() {
    UpdateLoggingConfigurationRequest request = new UpdateLoggingConfigurationRequest();
    request.setLevel(LoggerLevel.DEBUG);

    underTest.update("org.globalart.astrum", request);

    verify(logManager).setLoggerLevel("org.globalart.astrum", LoggerLevel.DEBUG);
  }

  @Test
  void update_unsetsLoggerLevel_whenDefault() {
    UpdateLoggingConfigurationRequest request = new UpdateLoggingConfigurationRequest();
    request.setLevel(LoggerLevel.DEFAULT);

    underTest.update("org.globalart.astrum", request);

    verify(logManager).unsetLoggerLevel("org.globalart.astrum");
  }

  @Test
  void update_throwsBadRequest_onIllegalArgument() {
    UpdateLoggingConfigurationRequest request = new UpdateLoggingConfigurationRequest();
    request.setLevel(LoggerLevel.TRACE);

    doThrow(new IllegalArgumentException("Invalid level"))
        .when(logManager)
        .setLoggerLevel("ROOT", LoggerLevel.TRACE);

    assertThrows(BadRequestException.class, () -> underTest.update("ROOT", request));
  }

  @Test
  void reset_unsetsLoggerLevel() {
    underTest.reset("org.globalart.astrum");

    verify(logManager).unsetLoggerLevel("org.globalart.astrum");
  }

  @Test
  void update_withInfoLevel() {
    UpdateLoggingConfigurationRequest request = new UpdateLoggingConfigurationRequest();
    request.setLevel(LoggerLevel.INFO);

    underTest.update("com.example", request);

    verify(logManager).setLoggerLevel("com.example", LoggerLevel.INFO);
  }

  @Test
  void update_withErrorLevel() {
    UpdateLoggingConfigurationRequest request = new UpdateLoggingConfigurationRequest();
    request.setLevel(LoggerLevel.ERROR);

    underTest.update("com.example", request);

    verify(logManager).setLoggerLevel("com.example", LoggerLevel.ERROR);
  }

  @Test
  void update_withWarnLevel() {
    UpdateLoggingConfigurationRequest request = new UpdateLoggingConfigurationRequest();
    request.setLevel(LoggerLevel.WARN);

    underTest.update("com.example", request);

    verify(logManager).setLoggerLevel("com.example", LoggerLevel.WARN);
  }

  @Test
  void update_withTraceLevel() {
    UpdateLoggingConfigurationRequest request = new UpdateLoggingConfigurationRequest();
    request.setLevel(LoggerLevel.TRACE);

    underTest.update("com.example", request);

    verify(logManager).setLoggerLevel("com.example", LoggerLevel.TRACE);
  }

  @Test
  void readAll_correctlyMapsLoggerLevels() {
    Map<String, LoggerLevel> loggers = new LinkedHashMap<>();
    loggers.put("ROOT", LoggerLevel.INFO);
    loggers.put("org.globalart", LoggerLevel.DEBUG);
    loggers.put("com.example", LoggerLevel.WARN);
    when(logManager.getEffectiveLoggersUpdatedByFetchedOverrides()).thenReturn(loggers);

    Collection<LoggerXO> result = underTest.readAll();

    assertThat(result, hasSize(3));

    boolean foundRoot = false;
    boolean foundGlobalArt = false;
    boolean foundExample = false;
    for (LoggerXO logger : result) {
      if ("ROOT".equals(logger.getName())) {
        assertThat(logger.getLevel(), is(LoggerLevel.INFO));
        foundRoot = true;
      }
      else if ("org.globalart".equals(logger.getName())) {
        assertThat(logger.getLevel(), is(LoggerLevel.DEBUG));
        foundGlobalArt = true;
      }
      else if ("com.example".equals(logger.getName())) {
        assertThat(logger.getLevel(), is(LoggerLevel.WARN));
        foundExample = true;
      }
    }
    assertThat(foundRoot, is(true));
    assertThat(foundGlobalArt, is(true));
    assertThat(foundExample, is(true));
  }
}
