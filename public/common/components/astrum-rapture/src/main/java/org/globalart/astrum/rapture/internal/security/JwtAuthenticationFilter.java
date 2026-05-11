
package org.globalart.astrum.rapture.internal.security;

import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.annotation.WebFilter;
import jakarta.servlet.http.Cookie;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.common.app.WebFilterPriority;
import org.globalart.astrum.security.JwtHelper;

import org.springframework.beans.factory.annotation.Autowired;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;

/**
 * JWT authentication filter for {@link JwtServlet}.
 *
 * @since 3.38
 */
@WebFilter(filterName = JwtAuthenticationFilter.NAME)
@Order(WebFilterPriority.AUTHENTICATION)
@Component
@ConditionalOnProperty(name = JWT_ENABLED, havingValue = "true")
public class JwtAuthenticationFilter
    extends SessionAuthenticationFilter
{
  private static final Logger log = LoggerFactory.getLogger(JwtAuthenticationFilter.class);

  public static final String NAME = "nx-jwt-authc";

  private final JwtHelper jwtHelper;

  @Autowired
  public JwtAuthenticationFilter(final JwtHelper jwtHelper) {
    this.jwtHelper = checkNotNull(jwtHelper);
  }

  @Override
  protected boolean onLoginSuccess(
      final AuthenticationToken token,
      final Subject subject,
      final ServletRequest request,
      final ServletResponse response) throws Exception
  {
    log.debug("Success: token={}, subject={}", token, subject);
    Cookie cookie = jwtHelper.createJwtCookie(subject, request.isSecure());
    ((HttpServletResponse) response).addCookie(cookie);
    return true;
  }
}
