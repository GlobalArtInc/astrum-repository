
package org.globalart.astrum.rapture.internal.settings;

import java.util.Optional;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.rapture.settings.RaptureSettings;
import org.globalart.astrum.security.JwtHelper;

import org.apache.shiro.astrum.AstrumWebSessionManager;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertEquals;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.anyLong;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;

/**
 * Tests for {@link SessionTimeoutManager}.
 */
@RunWith(MockitoJUnitRunner.class)
public class SessionTimeoutManagerTest
{
  @Mock
  private JwtHelper jwtHelper;

  @Mock
  private AstrumWebSessionManager sessionManager;

  @Mock
  private EventManager eventManager;

  @Test
  public void testPropertyOverride_jwtProperty() {
    // JWT property set to 600 seconds (10 minutes)
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.empty(),
        eventManager,
        600, // JWT property
        -1 // No session property
    );

    // Attempt to update via UI to 720 minutes
    manager.updateTimeout(720);

    // Verify property wins - no changes applied, no event posted
    verify(jwtHelper, never()).setExpirySeconds(anyInt());
    verify(eventManager, never()).post(any());
  }

  @Test
  public void testPropertyOverride_sessionProperty() {
    // Session property set to 600000 ms (10 minutes)
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.empty(),
        Optional.of(sessionManager),
        eventManager,
        -1, // No JWT property
        600000 // Session property
    );

    // Attempt to update via UI to 720 minutes
    manager.updateTimeout(720);

    // Verify property wins - no changes applied, no event posted
    verify(sessionManager, never()).setGlobalSessionTimeout(anyLong());
    verify(eventManager, never()).post(any());
  }

  @Test
  public void testUpdateTimeout_jwtMode_noPropertyOverride() {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.empty(),
        eventManager,
        -1, // No JWT property
        -1 // No session property
    );

    // Update to 720 minutes
    manager.updateTimeout(720);

    // Verify JWT updated: 720 minutes = 43200 seconds
    verify(jwtHelper).setExpirySeconds(43200);
    // Verify event posted
    ArgumentCaptor<SessionTimeoutChangedEvent> eventCaptor = ArgumentCaptor.forClass(SessionTimeoutChangedEvent.class);
    verify(eventManager).post(eventCaptor.capture());
    assertEquals(720, eventCaptor.getValue().getTimeoutMinutes());
  }

  @Test
  public void testUpdateTimeout_sessionMode_noPropertyOverride() {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.empty(),
        Optional.of(sessionManager),
        eventManager,
        -1, // No JWT property
        -1 // No session property
    );

    // Update to 720 minutes
    manager.updateTimeout(720);

    // Verify session updated: 720 minutes = 43200000 milliseconds
    verify(sessionManager).setGlobalSessionTimeout(43200000L);
    // Verify event posted
    ArgumentCaptor<SessionTimeoutChangedEvent> eventCaptor = ArgumentCaptor.forClass(SessionTimeoutChangedEvent.class);
    verify(eventManager).post(eventCaptor.capture());
    assertEquals(720, eventCaptor.getValue().getTimeoutMinutes());
  }

  @Test
  public void testUpdateTimeout_zeroTimeout_jwtMode() {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.empty(),
        eventManager,
        -1, // No property
        -1);

    // Move off the default of 0, then back to 0 minutes (never expire)
    manager.updateTimeout(60);
    manager.updateTimeout(0);

    // Verify JWT updated to 1 year (31536000 seconds)
    verify(jwtHelper).setExpirySeconds(31536000);
    // Verify event posted
    ArgumentCaptor<SessionTimeoutChangedEvent> eventCaptor = ArgumentCaptor.forClass(SessionTimeoutChangedEvent.class);
    verify(eventManager, times(2)).post(eventCaptor.capture());
    assertEquals(0, eventCaptor.getValue().getTimeoutMinutes());
  }

  @Test
  public void testUpdateTimeout_zeroTimeout_sessionMode() {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.empty(),
        Optional.of(sessionManager),
        eventManager,
        -1, // No property
        -1);

    // Move off the default of 0, then back to 0 minutes (never expire)
    manager.updateTimeout(60);
    manager.updateTimeout(0);

    // Verify session updated to -1 (Shiro convention for never expire)
    verify(sessionManager).setGlobalSessionTimeout(-1L);
    // Verify event posted
    ArgumentCaptor<SessionTimeoutChangedEvent> eventCaptor = ArgumentCaptor.forClass(SessionTimeoutChangedEvent.class);
    verify(eventManager, times(2)).post(eventCaptor.capture());
    assertEquals(0, eventCaptor.getValue().getTimeoutMinutes());
  }

  @Test
  public void testUpdateTimeout_sameValue_noEventPosted() {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.empty(),
        eventManager,
        -1, // No property
        -1);

    // Update to the default again
    manager.updateTimeout(RaptureSettings.DEFAULT_SESSION_TIMEOUT);

    // Verify no changes applied, no event posted (same value)
    verify(jwtHelper, never()).setExpirySeconds(anyInt());
    verify(eventManager, never()).post(any());
  }

  @Test
  public void testStart_appliesTheDefaultOfNeverExpire() throws Exception {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.of(sessionManager),
        eventManager,
        -1, // No property
        -1);

    manager.start();

    // The default of 0 means never expire: 1 year for JWT, -1 for Shiro
    verify(jwtHelper).setExpirySeconds(31536000);
    verify(sessionManager).setGlobalSessionTimeout(-1L);
    // Nothing changed, so there is nothing to tell other nodes
    verify(eventManager, never()).post(any());
  }

  @Test
  public void testStart_leavesPropertyOverrideAlone() throws Exception {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.of(sessionManager),
        eventManager,
        600, // JWT property set
        -1);

    manager.start();

    verify(jwtHelper, never()).setExpirySeconds(anyInt());
    verify(sessionManager, never()).setGlobalSessionTimeout(anyLong());
  }

  @Test
  public void testRemoteEvent_appliedWhenNoPropertyOverride() {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.empty(),
        eventManager,
        -1, // No property
        -1);

    // Simulate remote event from node-2
    SessionTimeoutChangedEvent remoteEvent = new SessionTimeoutChangedEvent(360);
    remoteEvent.setRemoteNodeId("node-2");

    manager.on(remoteEvent);

    // Verify timeout applied: 360 minutes = 21600 seconds
    verify(jwtHelper).setExpirySeconds(21600);
    // No event should be posted (this is a received event)
    verify(eventManager, never()).post(any());
  }

  @Test
  public void testRemoteEvent_ignoredWhenPropertyOverride() {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.empty(),
        eventManager,
        600, // JWT property set
        -1);

    // Simulate remote event from node-2
    SessionTimeoutChangedEvent remoteEvent = new SessionTimeoutChangedEvent(720);
    remoteEvent.setRemoteNodeId("node-2");

    manager.on(remoteEvent);

    // Verify property wins - remote event ignored
    verify(jwtHelper, never()).setExpirySeconds(anyInt());
    verify(eventManager, never()).post(any());
  }

  @Test
  public void testLocalEvent_skipped() {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.empty(),
        eventManager,
        -1, // No property
        -1);

    // Simulate local event (remoteNodeId is null)
    SessionTimeoutChangedEvent localEvent = new SessionTimeoutChangedEvent(360);

    manager.on(localEvent);

    // Verify local event is skipped - no changes applied
    verify(jwtHelper, never()).setExpirySeconds(anyInt());
    verify(eventManager, never()).post(any());
  }

  @Test
  public void testMultipleUpdates() {
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.empty(),
        eventManager,
        -1, // No property
        -1);

    // Update multiple times
    manager.updateTimeout(120);
    manager.updateTimeout(240);
    manager.updateTimeout(360);

    // Verify all updates applied
    verify(jwtHelper).setExpirySeconds(7200); // 120 minutes
    verify(jwtHelper).setExpirySeconds(14400); // 240 minutes
    verify(jwtHelper).setExpirySeconds(21600); // 360 minutes

    // Verify events posted for each update
    verify(eventManager, times(3)).post(any(SessionTimeoutChangedEvent.class));
  }

  @Test
  public void testBothModesPresent_onlyActiveOneUpdated() {
    // This shouldn't happen in practice, but test the behavior
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.of(jwtHelper),
        Optional.of(sessionManager),
        eventManager,
        -1, // No property
        -1);

    manager.updateTimeout(720);

    // Both should be updated
    verify(jwtHelper).setExpirySeconds(43200);
    verify(sessionManager).setGlobalSessionTimeout(43200000L);
    verify(eventManager).post(any(SessionTimeoutChangedEvent.class));
  }

  @Test
  public void testNoModePresent_noError() {
    // Edge case: neither JWT nor Session manager available
    SessionTimeoutManager manager = new SessionTimeoutManager(
        Optional.empty(),
        Optional.empty(),
        eventManager,
        -1, // No property
        -1);

    // Should not throw exception
    manager.updateTimeout(720);

    // Event should still be posted
    ArgumentCaptor<SessionTimeoutChangedEvent> eventCaptor = ArgumentCaptor.forClass(SessionTimeoutChangedEvent.class);
    verify(eventManager).post(eventCaptor.capture());
    assertEquals(720, eventCaptor.getValue().getTimeoutMinutes());
  }
}
