
package org.globalart.astrum.onboarding.internal;

import jakarta.servlet.http.Cookie;
import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.globalart.astrum.security.JwtHelper;

import org.apache.shiro.SecurityUtils;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.app.FeatureFlags.JWT_ENABLED;

/**
 * JWT-mode {@link OnboardingSessionRefresher}: mints a fresh JWT cookie so the caller's next
 * request presents a token with {@code iat >= cutoff} recorded by the preceding
 * {@link org.globalart.astrum.security.session.SessionInvalidator} call. {@code
 * JwtSecurityFilter} treats {@code iat == cutoff} as valid (strict {@code revoked_at > iat} in
 * {@code JwtSessionDAO.xml}), so no ordering workaround is required.
 */
@Component
@ConditionalOnProperty(name = JWT_ENABLED, havingValue = "true")
class JwtOnboardingSessionRefresher
    implements OnboardingSessionRefresher
{
  private static final Logger log = LoggerFactory.getLogger(JwtOnboardingSessionRefresher.class);

  private final JwtHelper jwtHelper;

  JwtOnboardingSessionRefresher(final JwtHelper jwtHelper) {
    this.jwtHelper = checkNotNull(jwtHelper);
  }

  @Override
  public void refreshIfSelfChange(
      final String userId,
      final String newPassword,
      final HttpServletRequest request,
      final HttpServletResponse response)
  {
    Subject subject = SecurityUtils.getSubject();
    if (subject == null || subject.getPrincipal() == null
        || !userId.equals(subject.getPrincipal().toString())) {
      return;
    }
    try {
      Cookie freshCookie = jwtHelper.createJwtCookie(subject, request.isSecure());
      response.addCookie(freshCookie);
      log.info("Issued fresh JWT for onboarding self-password-change by user '{}'", userId);
    }
    catch (Exception e) {
      log.warn("Failed to issue fresh JWT for onboarding self-password-change by user '{}': {}",
          userId, e.getMessage());
    }
  }
}
