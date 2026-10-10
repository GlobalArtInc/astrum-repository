
package org.globalart.astrum.security;

import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.util.Date;
import java.util.Optional;

import jakarta.servlet.ServletRequest;
import jakarta.servlet.ServletResponse;
import jakarta.servlet.annotation.WebFilter;
import jakarta.servlet.http.Cookie;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.common.text.Strings2;
import org.globalart.astrum.security.jwt.JwtSessionRevocationService;
import org.globalart.astrum.security.jwt.JwtVerificationException;

import com.auth0.jwt.interfaces.Claim;
import com.auth0.jwt.interfaces.DecodedJWT;
import org.apache.shiro.web.servlet.AdviceFilter;
import org.apache.shiro.web.util.WebUtils;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.Arrays.stream;
import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;
import static org.globalart.astrum.security.JwtHelper.JWT_COOKIE_NAME;
import static org.globalart.astrum.security.JwtHelper.USER;
import static org.globalart.astrum.security.JwtHelper.USER_SESSION_ID;

/**
 * Filter to verify and refresh JWT cookie
 *
 * @since 3.38
 */
@ConditionalOnProperty(name = JWT_ENABLED, havingValue = "true")
@WebFilter(filterName = JwtFilter.NAME)
@Component
public class JwtFilter
    extends AdviceFilter
{
  public static final String NAME = "as-jwt";

  private final JwtHelper jwtHelper;

  private final JwtSessionRevocationService jwtSessionRevocationService;

  @Autowired
  public JwtFilter(
      final JwtHelper jwtHelper,
      final JwtSessionRevocationService jwtSessionRevocationService)
  {
    this.jwtHelper = checkNotNull(jwtHelper);
    this.jwtSessionRevocationService = checkNotNull(jwtSessionRevocationService);
  }

  @Override
  protected boolean preHandle(final ServletRequest request, final ServletResponse response) throws Exception {
    HttpServletRequest servletRequest = (HttpServletRequest) request;
    Cookie[] cookies = servletRequest.getCookies();

    if (cookies != null) {
      Optional<Cookie> jwtCookie = stream(cookies)
          .filter(cookie -> cookie.getName().equals(JWT_COOKIE_NAME))
          .findFirst();

      if (jwtCookie.isPresent()) {
        Cookie cookie = jwtCookie.get();
        String jwt = cookie.getValue();
        if (!Strings2.isEmpty(jwt)) {
          DecodedJWT decoded;
          try {
            decoded = jwtHelper.verifyJwt(jwt);
          }
          catch (JwtVerificationException e) {
            // expire the cookie in case of any issues while JWT verification
            expireCookie(cookie, response);
            return false;
          }

          // Do not refresh a revoked session's JWT. Clearing here keeps the browser's
          // stored cookie from advancing its iat past a user-invalidation cutoff.
          if (isRevokedOrInvalidated(decoded)) {
            expireCookie(cookie, response);
            return false;
          }

          WebUtils.toHttp(response).addCookie(jwtHelper.refreshJwtCookie(decoded, request.isSecure()));
        }
      }
    }
    return true;
  }

  private boolean isRevokedOrInvalidated(final DecodedJWT decoded) {
    Claim sessionIdClaim = decoded.getClaim(USER_SESSION_ID);
    if (sessionIdClaim != null && !sessionIdClaim.isNull()
        && jwtSessionRevocationService.isRevoked(sessionIdClaim.asString())) {
      return true;
    }

    Claim userClaim = decoded.getClaim(USER);
    Date issuedAt = decoded.getIssuedAt();
    if (userClaim == null || userClaim.isNull() || issuedAt == null) {
      return false;
    }

    OffsetDateTime iat = issuedAt.toInstant().atOffset(ZoneOffset.UTC);
    return jwtSessionRevocationService.isUserInvalidatedAfter(userClaim.asString(), iat);
  }

  private static void expireCookie(final Cookie cookie, final ServletResponse response) {
    cookie.setValue("");
    cookie.setMaxAge(0);
    WebUtils.toHttp(response).addCookie(cookie);
  }
}
