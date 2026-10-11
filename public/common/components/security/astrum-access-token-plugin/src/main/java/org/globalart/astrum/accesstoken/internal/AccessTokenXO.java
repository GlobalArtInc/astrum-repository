package org.globalart.astrum.accesstoken.internal;

import java.time.OffsetDateTime;

/**
 * An access token as the UI sees it: never the secret, only its first characters.
 */
public record AccessTokenXO(
    String id,
    String name,
    String owner,
    String scope,
    String hint,
    OffsetDateTime created,
    OffsetDateTime expires,
    OffsetDateTime lastUsed,
    boolean expired)
{
  static AccessTokenXO from(final AccessTokenData token, final OffsetDateTime now) {
    return new AccessTokenXO(token.getId(), token.getName(), token.getUsername(), token.getScope(),
        token.getTokenHint() + "…", token.getCreated(), token.getExpires(), token.getLastUsed(), token.isExpired(now));
  }
}
