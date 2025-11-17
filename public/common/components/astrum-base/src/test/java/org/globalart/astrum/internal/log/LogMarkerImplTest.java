
package org.globalart.astrum.internal.log;

import java.util.EnumSet;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.log.LogManager;
import org.globalart.astrum.common.log.LogMarkInsertedEvent;
import org.globalart.astrum.common.log.LoggerLevel;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;

import static org.hamcrest.Matchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.ArgumentMatchers.isNotNull;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.reset;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

/**
 * Tests {@link LogMarkerImpl}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class LogMarkerImplTest
{
  private static final String LOG_NAME = LogMarkerImpl.class.getName();

  @Mock
  private LogManager logManager;

  @Mock
  private EventManager eventManager;

  private LogMarkerImpl logMarker;

  @Before
  public void setUp() {
    logMarker = new LogMarkerImpl(logManager, eventManager);
    when(logManager.getLoggerEffectiveLevel(LOG_NAME)).thenReturn(LoggerLevel.INFO);
  }

  @Test
  public void testMarkLog_EmitEvent() {
    logMarker.markLog("test");
    ArgumentCaptor<LogMarkInsertedEvent> argCaptor = ArgumentCaptor.forClass(LogMarkInsertedEvent.class);
    verify(eventManager).post(argCaptor.capture());
    assertThat(argCaptor.getValue().getMessage(), is("test"));
  }

  @Test
  public void testMarkLog_EnsureLogLevel() {
    for (LoggerLevel level : EnumSet.complementOf(EnumSet.of(LoggerLevel.DEFAULT))) {
      reset(logManager);
      when(logManager.getLoggerEffectiveLevel(LOG_NAME)).thenReturn(level);
      logMarker.markLog("test");
      try {
        if (EnumSet.of(LoggerLevel.OFF, LoggerLevel.ERROR, LoggerLevel.WARN).contains(level)) {
          verify(logManager).setLoggerLevel(LOG_NAME, LoggerLevel.INFO);
        }
        else {
          verify(logManager, never()).setLoggerLevel(isNotNull(), isNotNull());
        }
      }
      catch (AssertionError e) {
        throw new AssertionError("Mishandled log level " + level, e);
      }
    }
  }
}
