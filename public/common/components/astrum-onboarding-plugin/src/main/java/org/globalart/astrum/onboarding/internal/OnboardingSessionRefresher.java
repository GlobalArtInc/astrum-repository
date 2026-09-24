
package org.globalart.astrum.onboarding.internal;

import jakarta.servlet.http.HttpServletRequest;
import jakarta.servlet.http.HttpServletResponse;

/**
 * Keeps the caller's session alive after they change their own password through the onboarding
 * wizard. Every other session for the same user is invalidated by
 * {@link org.globalart.astrum.security.SecuritySystem#changePassword}'s internal
 * {@link org.globalart.astrum.security.session.SessionInvalidator} call — this refresher exempts
 * only the caller, and only from the onboarding endpoint.
 *
 * <p>
 * Package-private on purpose. The session invalidation on self-password-change is the intended
 * CVE fix from ASTRUM-52579 and must remain in force for User Account self-service, REST API,
 * ExtJS, and cloud copies. Do <strong>not</strong> wire this refresher into any other caller.
 */
interface OnboardingSessionRefresher
{
  /**
   * Mint a fresh session token on the response when the caller is the same user whose password
   * just changed. No-op when the caller is anonymous or is targeting a different user.
   *
   * @param userId the userId whose password just changed
   * @param newPassword the new password value (used by the Shiro implementation to
   *          re-authenticate the subject; ignored by the JWT implementation)
   * @param request the current servlet request (used to determine the cookie secure flag)
   * @param response the current servlet response (a fresh session cookie is attached here)
   */
  void refreshIfSelfChange(
      String userId,
      String newPassword,
      HttpServletRequest request,
      HttpServletResponse response);
}
