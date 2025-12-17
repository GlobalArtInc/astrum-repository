
package org.globalart.astrum.security.authc;

import java.util.List;

import javax.annotation.Nullable;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.http.HttpServletRequest;

import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.web.util.WebUtils;

/**
 * Support class for {@link AuthenticationTokenFactory}s that creates {@link AuthenticationToken}s based on HTTP
 * headers.
 *
 * Looks up given HTTP header names. If found will create an {@link HttpHeaderAuthenticationToken}.
 *
 * @since 2.7
 */
public abstract class HttpHeaderAuthenticationTokenFactorySupport
    implements AuthenticationTokenFactory
{
  @Override
  @Nullable
  public AuthenticationToken createToken(ServletRequest request, ServletResponse response) {
    List<String> headerNames = getHttpHeaderNames();
    if (headerNames != null) {
      HttpServletRequest httpRequest = WebUtils.toHttp(request);
      for (String headerName : headerNames) {
        String headerValue = httpRequest.getHeader(headerName);
        if (headerValue != null) {
          return createToken(headerName, headerValue, request.getRemoteHost());
        }
      }
    }
    return null;
  }

  /**
   * Creates the {@link HttpHeaderAuthenticationToken}. Subclasses can override and create specific tokens.
   */
  protected HttpHeaderAuthenticationToken createToken(String headerName, String headerValue, String host) {
    return new HttpHeaderAuthenticationToken(headerName, headerValue, host);
  }

  /**
   * Returns a list of HTTP header names that should be considered for creating the authentication tokens (should not
   * be null).
   */
  protected abstract List<String> getHttpHeaderNames();

  @Override
  public String toString() {
    return getClass().getSimpleName()
        + "(creates authentication tokens if any of HTTP headers is present: "
        + getHttpHeaderNames()
        + ")";
  }
}
