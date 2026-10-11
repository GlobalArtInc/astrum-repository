package org.globalart.astrum.accesstoken.internal;

import java.time.OffsetDateTime;

import javax.annotation.Nullable;

import org.globalart.astrum.security.accesstoken.AccessTokenScope;

import org.apache.shiro.subject.PrincipalCollection;

/**
 * A stored access token. Only a hash of the secret is kept.
 */
public class AccessTokenData
{
  private String id;

  private String username;

  private PrincipalCollection principals;

  private String name;

  private String tokenHash;

  private String tokenHint;

  private String scope;

  private OffsetDateTime created;

  private OffsetDateTime expires;

  private OffsetDateTime lastUsed;

  public String getId() {
    return id;
  }

  public void setId(final String id) {
    this.id = id;
  }

  public String getUsername() {
    return username;
  }

  public void setUsername(final String username) {
    this.username = username;
  }

  public PrincipalCollection getPrincipals() {
    return principals;
  }

  public void setPrincipals(final PrincipalCollection principals) {
    this.principals = principals;
  }

  public String getName() {
    return name;
  }

  public void setName(final String name) {
    this.name = name;
  }

  public String getTokenHash() {
    return tokenHash;
  }

  public void setTokenHash(final String tokenHash) {
    this.tokenHash = tokenHash;
  }

  public String getTokenHint() {
    return tokenHint;
  }

  public void setTokenHint(final String tokenHint) {
    this.tokenHint = tokenHint;
  }

  public String getScope() {
    return scope;
  }

  public void setScope(final String scope) {
    this.scope = scope;
  }

  public AccessTokenScope scope() {
    return AccessTokenScope.fromId(scope);
  }

  public OffsetDateTime getCreated() {
    return created;
  }

  public void setCreated(final OffsetDateTime created) {
    this.created = created;
  }

  @Nullable
  public OffsetDateTime getExpires() {
    return expires;
  }

  public void setExpires(@Nullable final OffsetDateTime expires) {
    this.expires = expires;
  }

  @Nullable
  public OffsetDateTime getLastUsed() {
    return lastUsed;
  }

  public void setLastUsed(@Nullable final OffsetDateTime lastUsed) {
    this.lastUsed = lastUsed;
  }

  public boolean isExpired(final OffsetDateTime now) {
    return expires != null && !expires.isAfter(now);
  }
}
