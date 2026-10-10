
package org.globalart.astrum.security.authc;

import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.annotation.WebFilter;

import org.globalart.astrum.common.app.WebFilterPriority;

import com.google.common.collect.Lists;
import org.apache.shiro.authc.AuthenticationToken;
import org.apache.shiro.web.filter.authc.AuthenticatingFilter;
import org.apache.shiro.web.filter.authc.BasicHttpAuthenticationFilter;
import org.springframework.core.annotation.Order;
import org.springframework.lang.Nullable;

// FIXME: Rename this class, unsure what but its confusing asis
// FIXME: Also like api-key filter, this isn't basic-auth, but is extending from that base

/**
 * {@link AuthenticatingFilter} that delegates token creation to {@link AuthenticationTokenFactory}s before falling
 * back to {@link BasicHttpAuthenticationFilter}.
 *
 * e.g. {@link AuthenticationTokenFactory} that will lookup REMOTE_USER HTTP header
 *
 * @since 2.7
 */
@WebFilter(filterName = AstrumAuthenticationFilter.NAME)
@Order(WebFilterPriority.AUTHENTICATION)
public class AstrumAuthenticationFilter
    extends AstrumBasicHttpAuthenticationFilter
{
  public static final String NAME = "as-authc";

  private List<AuthenticationTokenFactory> factories = Lists.newArrayList();

  @Autowired
  public void install(@Nullable final List<AuthenticationTokenFactory> factories) {
    this.factories = factories == null ? List.of() : factories;
  }

  /**
   * Will consider an login attempt if any of the factories is able to create an authentication token.
   *
   * Otherwise will fallback to {@link BasicHttpAuthenticationFilter#isLoginAttempt(ServletRequest, ServletResponse)}
   */
  @Override
  protected boolean isLoginAttempt(final ServletRequest request, final ServletResponse response) {
    AuthenticationToken token = createAuthenticationToken(request, response);
    return token != null || super.isLoginAttempt(request, response);
  }

  /**
   * Will cycle configured factories for an authentication token. First one that will return a non null one will win.
   * If none of them will return an authentication token will fallback to
   * {@link BasicHttpAuthenticationFilter#createToken(ServletRequest, ServletResponse)}
   */
  @Override
  protected AuthenticationToken createToken(final ServletRequest request, final ServletResponse response) {
    AuthenticationToken token = createAuthenticationToken(request, response);
    if (token != null) {
      return token;
    }
    return super.createToken(request, response);
  }

  private AuthenticationToken createAuthenticationToken(final ServletRequest request, final ServletResponse response) {
    for (AuthenticationTokenFactory factory : factories) {
      try {
        AuthenticationToken token = factory.createToken(request, response);
        if (token != null) {
          log.debug("Token '{}' created by {}", token, factory);
          return token;
        }
      }
      catch (Exception e) {
        log.warn(
            "Factory {} failed to create an authentication token {}/{}",
            factory, e.getClass().getName(), e.getMessage(),
            log.isDebugEnabled() ? e : null);
      }
    }
    return null;
  }
}
