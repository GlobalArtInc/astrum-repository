
package org.globalart.astrum.rapture.internal.settings;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertFalse;
import static org.junit.jupiter.api.Assertions.assertTrue;

/**
 * Tests for {@link SessionTimeoutChangedEvent}.
 */
class SessionTimeoutChangedEventTest
{
  @Test
  void testIsLocal_whenRemoteNodeIdIsNull() {
    SessionTimeoutChangedEvent event = new SessionTimeoutChangedEvent(120);
    assertTrue(event.isLocal(), "Event should be local when remoteNodeId is null");
  }

  @Test
  void testIsLocal_whenRemoteNodeIdIsSet() {
    SessionTimeoutChangedEvent event = new SessionTimeoutChangedEvent(120);
    event.setRemoteNodeId("node-2");
    assertFalse(event.isLocal(), "Event should not be local when remoteNodeId is set");
  }
}
