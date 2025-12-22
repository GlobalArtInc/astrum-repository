
package org.globalart.astrum.security.session;

/**
 * Interface for invalidating user sessions.
 * Implementations are conditionally loaded based on JWT/Session feature flags.
 */
public interface SessionInvalidator
{
  /**
   * Invalidates all active sessions for the given user.
   *
   * @param username the username whose sessions should be invalidated
   * @param userSource the user source/realm (e.g., "default", "LDAP", "SAML"); consulted
   *          by the JWT implementation to scope the invalidation to the correct realm.
   *          Session-mode implementations may ignore this.
   * @param reason human-readable reason for the invalidation (e.g., "password change",
   *          "user deletion", "user deactivation"); surfaced in log lines and audit records
   *          so operators can distinguish invalidation triggers.
   * @return the number of sessions (or invalidation markers) that were recorded
   */
  int invalidateSessionsForUser(String username, String userSource, String reason);
}
