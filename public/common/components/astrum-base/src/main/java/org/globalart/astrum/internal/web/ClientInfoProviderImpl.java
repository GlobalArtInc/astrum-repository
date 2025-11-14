
package org.globalart.astrum.internal.web;

import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.ws.rs.core.Context;

import org.globalart.astrum.security.ClientInfo;
import org.globalart.astrum.security.ClientInfoProvider;
import org.globalart.astrum.security.UserIdHelper;

import com.google.common.net.HttpHeaders;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

/**
 * Default {@link ClientInfoProvider}
 *
 * @since 3.0
 */
@Component
public class ClientInfoProviderImpl
    implements ClientInfoProvider
{
  private final Provider<HttpServletRequest> httpRequestProvider;

  private final ThreadLocal<String> remoteIp = new ThreadLocal<>();

  private final ThreadLocal<String> userId = new ThreadLocal<>();

  @Autowired
  public ClientInfoProviderImpl(@Context final Provider<HttpServletRequest> httpRequestProvider) {
    this.httpRequestProvider = checkNotNull(httpRequestProvider);
  }

  @Override
  @Nullable
  public ClientInfo getCurrentThreadClientInfo() {
    try {
      HttpServletRequest request = httpRequestProvider.get();
      return ClientInfo
          .builder()
          .userId(UserIdHelper.get())
          .remoteIP(request.getRemoteAddr())
          .userAgent(request.getHeader(HttpHeaders.USER_AGENT))
          .path(request.getServletPath())
          .build();
    }
    // TODO verify request type
    catch (Exception e) {
      /*
       * This happens when called out of scope of http request.
       * Create fake ClientInfo with the custom User Id and Remote address.
       */
      return userId.get() != null && remoteIp.get() != null
          ? ClientInfo
              .builder()
              .userId(userId.get())
              .remoteIP(remoteIp.get())
              .build()
          : null;
    }
  }

  @Override
  public void setClientInfo(final String remoteIp, final String userId) {
    this.remoteIp.set(checkNotNull(remoteIp));
    this.userId.set(checkNotNull(userId));
  }

  @Override
  public void unsetClientInfo() {
    remoteIp.remove();
    userId.remove();
  }
}
