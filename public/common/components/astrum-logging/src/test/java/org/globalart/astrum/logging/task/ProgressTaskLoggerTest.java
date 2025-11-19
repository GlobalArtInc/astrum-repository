
package org.globalart.astrum.logging.task;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.slf4j.Logger;
import org.slf4j.Marker;

import static java.lang.String.format;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyCollection;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.globalart.astrum.logging.task.ProgressTaskLogger.PROGRESS_LINE;
import static org.globalart.astrum.logging.task.TaskLoggingMarkers.INTERNAL_PROGRESS;

@RunWith(MockitoJUnitRunner.Silent.class)
public class ProgressTaskLoggerTest

{
  @Mock
  private Logger mockLogger;

  private ProgressTaskLogger underTest;

  @Before
  public void setUp() throws Exception {
    underTest = new ProgressTaskLogger(mockLogger);
    underTest.start();
  }

  @After
  public void tearDown() throws Exception {
    underTest.finish();
  }

  @Test
  public void testProgress() {
    String message = "test message";
    TaskLoggingEvent event = new TaskLoggingEvent(mockLogger, message);
    underTest.progress(event);

    // invoke method normally invoke via thread
    underTest.logProgress();

    // verify progress logged properly
    verifyLog(INTERNAL_PROGRESS, format(PROGRESS_LINE, message), (Object[]) null);
  }

  @Test
  public void testNoProgressLogged() {
    // invoke method normally invoke via thread
    underTest.logProgress();

    // verify no progress was logged as no progress message was set
    verify(mockLogger, never()).info(any(Marker.class), anyString(), anyCollection());
  }

  private void verifyLog(final Marker m, final String s, final Object... args) {
    verify(mockLogger).info(m, s, args);
  }
}
