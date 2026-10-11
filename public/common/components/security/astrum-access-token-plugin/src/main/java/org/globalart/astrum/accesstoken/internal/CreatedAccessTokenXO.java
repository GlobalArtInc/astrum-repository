package org.globalart.astrum.accesstoken.internal;

/**
 * A new access token together with its secret, which is never shown again.
 */
public record CreatedAccessTokenXO(AccessTokenXO token, String secret)
{
}
