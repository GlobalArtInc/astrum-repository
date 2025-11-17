
package org.globalart.astrum.internal.event;

import org.globalart.astrum.common.event.EventManager;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.reset;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;

/**
 * Tests for {@link DebugEventInspector}'s conditional event-bus register/unregister
 * dispatch. Covers state transitions in both directions plus idempotency in both states.
 */
@ExtendWith(MockitoExtension.class)
class DebugEventInspectorTest
{
  @Mock
  private EventManager eventManager;

  private DebugEventInspector underTest;

  @BeforeEach
  void setup() {
    underTest = new DebugEventInspector(eventManager);
    reset(eventManager);
  }

  @Test
  void shouldNotInteractWithEventManagerDuringConstructionWhenDefaultDisabled() {
    DebugEventInspector freshlyConstructed = new DebugEventInspector(eventManager);

    verifyNoInteractions(eventManager);
    assertThat(freshlyConstructed.isEnabled(), is(false));
  }

  @Test
  void shouldRegisterWithEventManagerWhenEnabledFromDisabledState() {
    underTest.setEnabled(true);

    verify(eventManager).register(underTest);
    verify(eventManager, never()).unregister(underTest);
    assertThat(underTest.isEnabled(), is(true));
  }

  @Test
  void shouldUnregisterFromEventManagerWhenDisabledFromEnabledState() {
    underTest.setEnabled(true);
    reset(eventManager);

    underTest.setEnabled(false);

    verify(eventManager).unregister(underTest);
    verify(eventManager, never()).register(underTest);
    assertThat(underTest.isEnabled(), is(false));
  }

  @Test
  void shouldNotRegisterAgainWhenAlreadyEnabled() {
    underTest.setEnabled(true);
    reset(eventManager);

    underTest.setEnabled(true);

    verifyNoInteractions(eventManager);
    assertThat(underTest.isEnabled(), is(true));
  }

  @Test
  void shouldNotUnregisterWhenAlreadyDisabled() {
    underTest.setEnabled(false);

    verifyNoInteractions(eventManager);
    assertThat(underTest.isEnabled(), is(false));
  }
}
