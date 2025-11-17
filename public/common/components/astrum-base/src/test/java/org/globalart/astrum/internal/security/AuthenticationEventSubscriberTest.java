
package org.globalart.astrum.internal.security;

import java.util.Collections;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.ClientInfo;
import org.globalart.astrum.security.ClientInfoProvider;
import org.globalart.astrum.security.authc.AuthenticationEvent;
import org.globalart.astrum.security.authc.AuthenticationFailureReason;
import org.globalart.astrum.security.authc.AstrumAuthenticationEvent;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.core.Is.is;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AuthenticationEventSubscriberTest
{
  @Mock
  private ClientInfoProvider clientInfoProvider;

  @Mock
  private EventManager eventManager;

  @Captor
  private ArgumentCaptor<Object> eventArgumentCaptor;

  private AuthenticationEventSubscriber authenticationEventSubscriber;

  @Before
  public void setup() {
    authenticationEventSubscriber = new AuthenticationEventSubscriber(() -> eventManager, () -> clientInfoProvider);
  }

  @Test
  public void testOn_successWithClientInfo() {
    ClientInfo clientInfo =
        ClientInfo.builder()
            .path("/path/to/artifact")
            .remoteIP("remote ip")
            .userAgent("user agent string")
            .userId("user id")
            .build();
    when(clientInfoProvider.getCurrentThreadClientInfo()).thenReturn(clientInfo);
    AuthenticationEvent authenticationEvent = new AuthenticationEvent("user id", true);

    authenticationEventSubscriber.on(authenticationEvent);

    verify(eventManager).post(eventArgumentCaptor.capture());

    AstrumAuthenticationEvent astrumAuthenticationEvent = (AstrumAuthenticationEvent) eventArgumentCaptor.getValue();
    assertThat(astrumAuthenticationEvent.getAuthenticationFailureReasons().isEmpty(), is(true));
    assertThat(astrumAuthenticationEvent.isSuccessful(), is(true));
    assertThat(astrumAuthenticationEvent.getClientInfo(), is(clientInfo));
  }

  @Test
  public void testOn_successWithoutClientInfo() {
    AuthenticationEvent authenticationEvent = new AuthenticationEvent("user id", true);

    authenticationEventSubscriber.on(authenticationEvent);

    verify(eventManager).post(eventArgumentCaptor.capture());

    AstrumAuthenticationEvent astrumAuthenticationEvent = (AstrumAuthenticationEvent) eventArgumentCaptor.getValue();
    assertThat(astrumAuthenticationEvent.getAuthenticationFailureReasons().isEmpty(), is(true));
    assertThat(astrumAuthenticationEvent.isSuccessful(), is(true));
    assertThat(astrumAuthenticationEvent.getClientInfo().getUserid(), is(authenticationEvent.getUserId()));
  }

  @Test
  public void testOn_failedEvent() {
    AuthenticationEvent authenticationEvent = new AuthenticationEvent("user id", false,
        Collections.singleton(AuthenticationFailureReason.INCORRECT_CREDENTIALS));

    authenticationEventSubscriber.on(authenticationEvent);

    verify(eventManager).post(eventArgumentCaptor.capture());

    AstrumAuthenticationEvent astrumAuthenticationEvent = (AstrumAuthenticationEvent) eventArgumentCaptor.getValue();
    assertThat(astrumAuthenticationEvent.getAuthenticationFailureReasons().size(), is(1));
    assertThat(astrumAuthenticationEvent.isSuccessful(), is(false));
    assertThat(astrumAuthenticationEvent.getAuthenticationFailureReasons(),
        contains(AuthenticationFailureReason.INCORRECT_CREDENTIALS));
  }
}
