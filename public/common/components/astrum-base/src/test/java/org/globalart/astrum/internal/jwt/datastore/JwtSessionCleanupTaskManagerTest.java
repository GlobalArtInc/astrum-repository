
package org.globalart.astrum.internal.jwt.datastore;

import java.time.Duration;

import org.globalart.astrum.common.scheduling.PeriodicJobService;
import org.globalart.astrum.common.scheduling.PeriodicJobService.PeriodicJob;
import org.globalart.astrum.security.jwt.JwtSessionRevocationService;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mock;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class JwtSessionCleanupTaskManagerTest
{
  @Mock
  private PeriodicJobService periodicJobService;

  @Mock
  private JwtSessionRevocationService jwtSessionRevocationService;

  private JwtSessionCleanupTaskManager underTest;

  @Before
  public void setup() {
    underTest = new JwtSessionCleanupTaskManager(periodicJobService, jwtSessionRevocationService);
  }

  @Test
  public void testStart() throws Exception {
    PeriodicJob mockJob = mock(PeriodicJob.class);
    when(periodicJobService.schedule(any(Runnable.class), any(Duration.class))).thenReturn(mockJob);

    underTest.doStart();

    verify(periodicJobService).startUsing();
    verify(periodicJobService).schedule(any(Runnable.class), eq(Duration.ofHours(24)));
  }

  @Test
  public void testStop() throws Exception {
    PeriodicJob mockJob = mock(PeriodicJob.class);
    when(periodicJobService.schedule(any(Runnable.class), any(Duration.class))).thenReturn(mockJob);

    underTest.doStart();
    underTest.doStop();

    verify(mockJob).cancel();
    verify(periodicJobService).stopUsing();
  }

  @Test
  public void testCleanupExpiredSessions() {
    when(jwtSessionRevocationService.deleteExpiredSessions()).thenReturn(5);

    underTest.cleanupExpiredSessions();

    verify(jwtSessionRevocationService).deleteExpiredSessions();
  }

  @Test
  public void testCleanupExpiredSessionsNoRecords() {
    when(jwtSessionRevocationService.deleteExpiredSessions()).thenReturn(0);

    underTest.cleanupExpiredSessions();

    verify(jwtSessionRevocationService).deleteExpiredSessions();
  }

  @Test
  public void testCleanupExpiredSessionsHandlesException() {
    when(jwtSessionRevocationService.deleteExpiredSessions())
        .thenThrow(new RuntimeException("Database error"));

    // Should not throw, but log warning
    underTest.cleanupExpiredSessions();

    verify(jwtSessionRevocationService).deleteExpiredSessions();
  }
}
