
package org.globalart.astrum.security.jwt;

import java.time.OffsetDateTime;

/**
 * Service for managing JWT session revocations.
 */
public interface JwtSessionRevocationService
{
  /**
   * Revoke a JWT session by recording its userSessionId.
   *
   * @param userSessionId the unique session identifier from JWT claim
   * @param username the username associated with the session
   * @param userSource the user source/realm (e.g., "default", "LDAP", "SAML")
   * @param expiresAt when the JWT expires
   */
  void revokeSession(String userSessionId, String username, String userSource, OffsetDateTime expiresAt);

  /**
   * Check if a JWT session is revoked.
   *
   * @param userSessionId the unique session identifier from JWT claim
   * @return true if the session is revoked, false otherwise
   */
  boolean isRevoked(String userSessionId);

  /**
   * Delete expired session revocations that are past their JWT expiration time.
   * This should be called periodically to prevent table bloat.
   *
   * @return the number of expired revocations deleted
   */
  int deleteExpiredSessions();

  /**
   * Record a cutoff that globally invalidates every JWT session for {@code username}, across all
   * realms the user may be authenticating through. After this call, any JWT for {@code username}
   * whose {@code iat} is at or before {@code cutoff} is treated as invalidated by
   * {@link #isUserInvalidatedAfter(String, OffsetDateTime)}, regardless of the realm the JWT was
   * issued from.
   *
   * @param username the user whose JWT sessions are being globally invalidated
   * @param userSource the user source/realm as known to the caller (e.g. {@code user.getSource()});
   *          stored on the invalidation row for audit/forensic purposes only — it is not
   *          used when matching JWTs against the cutoff
   * @param cutoff the "not valid before" timestamp (typically {@code now()})
   * @param validUntil when this invalidation row is safe to drop (typically
   *          {@code cutoff + maxJwtLifetime}); drives cleanup by the periodic job
   */
  void invalidateUser(String username, String userSource, OffsetDateTime cutoff, OffsetDateTime validUntil);

  /**
   * Check whether {@code username} has been globally invalidated after the JWT was issued.
   *
   * <p>
   * A password change globally invalidates every JWT session for the given username, across
   * all realms. This method returns {@code true} if any invalidation row exists for {@code username}
   * whose cutoff is strictly later than {@code iat} — the realm the JWT was issued from is not
   * considered.
   *
   * @param username the username from the JWT
   * @param iat the JWT issued-at timestamp
   * @return true if the username has been invalidated after the JWT was issued
   */
  boolean isUserInvalidatedAfter(String username, OffsetDateTime iat);
}
