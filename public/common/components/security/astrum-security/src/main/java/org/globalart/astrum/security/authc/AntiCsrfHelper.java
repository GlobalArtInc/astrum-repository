
package org.globalart.astrum.security.authc;

import java.util.Optional;

import jakarta.servlet.http.Cookie;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.ws.rs.HttpMethod;

import com.google.common.annotations.VisibleForTesting;
import com.google.common.net.HttpHeaders;
import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

/**
 * @since 3.16
 */
@Component
public class AntiCsrfHelper
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final String ENABLED = "astrum.security.anticsrftoken.enabled";

  public static final String SEC_FETCH_SITE_HEADER_ENABLED = "astrum.security.anticsrftoken.secFetchSite.enabled";

  public static final String ERROR_MESSAGE_TOKEN_MISMATCH = "Anti cross-site request forgery token mismatch";

  public static final String ANTI_CSRF_TOKEN_NAME = "AS-ANTI-CSRF-TOKEN";

  private final boolean enabled;

  private final boolean secFetchHeaderEnabled;

  @Autowired
  public AntiCsrfHelper(
      @Value("${" + ENABLED + ":true}") final boolean enabled,
      @Value("${" + SEC_FETCH_SITE_HEADER_ENABLED + ":true}") final boolean secFetchHeaderEnabled)
  {
    this.enabled = enabled;
    this.secFetchHeaderEnabled = secFetchHeaderEnabled;
  }

  /**
   * Checks the request for CSRF if the token is invalid.
   *
   * @return true if the token is valid or if the token does not require validation.
   */
  public boolean isAccessAllowed(final HttpServletRequest httpRequest) {
    if (!enabled) {
      return true;
    }
    boolean safeHttpMethod = isSafeHttpMethod(httpRequest);
    if (!safeHttpMethod && isCrossSiteRequest(httpRequest)) {
      log.debug("Blocking cross-site request header Sec-Fetch-Site:{}",
          httpRequest.getHeader(HttpHeaders.SEC_FETCH_SITE));
      return false;
    }

    return safeHttpMethod
        || !isSessionAuthentication() // non-session auth
        || isAntiCsrfTokenValid(httpRequest, Optional.ofNullable(httpRequest.getHeader(ANTI_CSRF_TOKEN_NAME)));
  }

  @VisibleForTesting
  boolean isCrossSiteRequest(final HttpServletRequest request) {
    if (!secFetchHeaderEnabled) {
      return false;
    }
    String secFetchSiteHeader = request.getHeader(HttpHeaders.SEC_FETCH_SITE);
    if (secFetchSiteHeader == null) {
      return false;
    }

    return switch (secFetchSiteHeader) {
      case "same-origin" -> false;
      case "none" -> false;
      default -> true;
    };
  }

  private static boolean isSafeHttpMethod(final HttpServletRequest request) {
    String method = request.getMethod();
    return HttpMethod.GET.equals(method) || HttpMethod.HEAD.equals(method);
  }

  private static boolean isSessionAuthentication() {
    Subject subject = SecurityUtils.getSubject();

    return subject != null && subject.getSession(false) != null;
  }

  /**
   * A browser may hold several cookies of the same name, for instance one per path or one set for the host and one
   * for its domain, and sends all of them. The token is valid when it matches any of them: each is a value only this
   * site could have put there.
   */
  private static boolean isAntiCsrfTokenValid(final HttpServletRequest request, final Optional<String> token) {
    if (token.isEmpty() || token.get().isEmpty()) {
      return false;
    }
    Cookie[] cookies = request.getCookies();
    if (cookies != null) {
      for (Cookie cookie : cookies) {
        if (ANTI_CSRF_TOKEN_NAME.equals(cookie.getName()) && token.get().equals(cookie.getValue())) {
          return true;
        }
      }
    }
    return false;
  }

  /**
   * @return whether CSRF protection is enabled
   */
  public boolean isEnabled() {
    return enabled;
  }
}
