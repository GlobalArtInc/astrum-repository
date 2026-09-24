
package org.globalart.astrum.onboarding.internal;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

import org.apache.shiro.SecurityUtils;
import org.apache.shiro.authc.UsernamePasswordToken;
import org.apache.shiro.subject.Subject;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.FeatureFlags.SESSION_ENABLED;

/**
 * Shiro-mode {@link OnboardingSessionRefresher}: re-authenticates the current Shiro subject with
 * the new password so Shiro creates a fresh session and sets the JSESSIONID cookie on the
 * response.
 */
@Component
@ConditionalOnProperty(name = SESSION_ENABLED, havingValue = "true", matchIfMissing = true)
class ShiroOnboardingSessionRefresher
    implements OnboardingSessionRefresher
{
  private static final Logger log = LoggerFactory.getLogger(ShiroOnboardingSessionRefresher.class);

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
      subject.login(new UsernamePasswordToken(userId, newPassword));
      log.info("Refreshed Shiro session for onboarding self-password-change by user '{}'", userId);
    }
    catch (Exception e) {
      log.warn("Failed to refresh Shiro session for onboarding self-password-change by user '{}': {}",
          userId, e.getMessage());
    }
  }
}
