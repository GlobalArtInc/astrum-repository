
package org.globalart.astrum.internal.security;

import com.google.common.eventbus.AllowConcurrentEvents;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.security.ClientInfo;
import org.globalart.astrum.security.ClientInfoProvider;
import org.globalart.astrum.security.authc.AuthenticationEvent;
import org.globalart.astrum.security.authc.AstrumAuthenticationEvent;

import com.google.common.eventbus.Subscribe;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Forwards {@link AuthenticationEvent} as {@link AstrumAuthenticationEvent}.
 *
 * @since 3.0
 */
@Component
public class AuthenticationEventSubscriber
    implements EventAware
{
  private final Provider<EventManager> eventManager;

  private final Provider<ClientInfoProvider> clientInfoProvider;

  @Autowired
  public AuthenticationEventSubscriber(
      final Provider<EventManager> eventManager,
      final Provider<ClientInfoProvider> clientInfoProvider)
  {
    this.eventManager = checkNotNull(eventManager);
    this.clientInfoProvider = checkNotNull(clientInfoProvider);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void on(final AuthenticationEvent event) {
    final ClientInfo clientInfo = clientInfoProvider.get().getCurrentThreadClientInfo();

    ClientInfo.Builder builder = ClientInfo
        .builder()
        .userId(event.getUserId());

    if (clientInfo != null) {
      builder
          .remoteIP(clientInfo.getRemoteIP())
          .userAgent(clientInfo.getUserAgent())
          .path(clientInfo.getPath());
    }

    eventManager.get()
        .post(new AstrumAuthenticationEvent(
            builder.build(),
            event.isSuccessful(),
            event.getAuthenticationFailureReasons()));
  }
}
