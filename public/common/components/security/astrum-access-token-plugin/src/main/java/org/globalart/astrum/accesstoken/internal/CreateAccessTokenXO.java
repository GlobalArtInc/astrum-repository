package org.globalart.astrum.accesstoken.internal;

/**
 * A request to create an access token; {@code expiresInDays} is empty for a token that never expires.
 */
public record CreateAccessTokenXO(String name, String scope, Integer expiresInDays)
{
}
