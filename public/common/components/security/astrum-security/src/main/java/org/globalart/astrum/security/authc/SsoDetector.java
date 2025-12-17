
package org.globalart.astrum.security.authc;

/**
 * Abstraction that allows filters in {@code astrum-rapture} to detect whether SSO is active
 * without creating a direct dependency on private SAML or OAuth2 modules.
 *
 * <p>
 * Implementations are registered as Spring beans. If no implementation is present
 * (e.g. in open-source builds), callers should treat SSO as disabled.
 * </p>
 */
public interface SsoDetector
{

  /**
   * Returns {@code true} if at least one SSO provider (SAML, OIDC/OAuth2, etc.) is currently
   * enabled and should receive the authentication traffic instead of local credentials.
   */
  boolean isSsoEnabled();
}
