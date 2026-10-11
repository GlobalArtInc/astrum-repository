package org.globalart.astrum.accesstoken.internal;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Posted when an access token is created or revoked.
 */
public class AccessTokenEvent
{
  public enum Type
  {
    CREATED,
    REVOKED
  }

  private final Type type;

  private final AccessTokenData token;

  public AccessTokenEvent(final Type type, final AccessTokenData token) {
    this.type = checkNotNull(type);
    this.token = checkNotNull(token);
  }

  public Type getType() {
    return type;
  }

  public AccessTokenData getToken() {
    return token;
  }
}
