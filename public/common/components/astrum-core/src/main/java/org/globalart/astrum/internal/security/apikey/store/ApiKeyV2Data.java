
package org.globalart.astrum.internal.security.apikey.store;

import java.time.OffsetDateTime;

import javax.annotation.Nullable;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.internal.security.apikey.ApiKeyInternal;

import org.apache.shiro.subject.PrincipalCollection;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * An {@link ApiKeyInternal} data for use with {@link ApiKeyStoreV2Impl}
 */
public class ApiKeyV2Data
    implements ApiKeyInternal
{
  private PrincipalCollection principals;

  private String username;

  private String domain;

  private String accessKey;

  private Secret secret;

  private OffsetDateTime created;

  private String createdByUserId;

  private String createdByRealm;

  ApiKeyV2Data() {
  }

  ApiKeyV2Data(
      final String domain,
      final PrincipalCollection principals,
      final String accessKey,
      final Secret secret,
      @Nullable final OffsetDateTime created)
  {
    this(domain, principals, accessKey, secret, created, null, null);
  }

  ApiKeyV2Data(
      final String domain,
      final PrincipalCollection principals,
      final String accessKey,
      final Secret secret,
      @Nullable final OffsetDateTime created,
      @Nullable final String createdByUserId,
      @Nullable final String createdByRealm)
  {
    this.domain = checkNotNull(domain);
    this.principals = checkNotNull(principals);
    this.username = principals.getPrimaryPrincipal().toString();
    this.accessKey = checkNotNull(accessKey);
    this.secret = checkNotNull(secret);
    this.created = created;
    this.createdByUserId = createdByUserId;
    this.createdByRealm = createdByRealm;
  }

  public String getAccessKey() {
    return accessKey;
  }

  @Override
  public char[] getApiKey() {
    int keyLength = accessKey.length();
    char[] secretPart = secret.decrypt();
    char[] token = new char[keyLength + secretPart.length];

    System.arraycopy(accessKey.toCharArray(), 0, token, 0, keyLength);
    System.arraycopy(secretPart, 0, token, keyLength, secretPart.length);

    return token;
  }

  @Override
  public OffsetDateTime getCreated() {
    return created;
  }

  @Override
  public String getDomain() {
    return domain;
  }

  @Override
  public PrincipalCollection getPrincipals() {
    return principals;
  }

  public Secret getSecret() {
    return secret;
  }

  public String getUsername() {
    return username;
  }

  public void setAccessKey(final String accessKey) {
    this.accessKey = checkNotNull(accessKey);
  }

  @Override
  public void setCreated(final OffsetDateTime created) {
    this.created = created;
  }

  @Override
  public void setDomain(final String domain) {
    this.domain = domain;
  }

  @Override
  public void setPrincipals(final PrincipalCollection principals) {
    this.principals = checkNotNull(principals);
    this.username = principals.getPrimaryPrincipal().toString();
  }

  public void setSecret(final Secret secret) {
    this.secret = secret;
  }

  public String getCreatedByUserId() {
    return createdByUserId;
  }

  public void setCreatedByUserId(final String createdByUserId) {
    this.createdByUserId = createdByUserId;
  }

  public String getCreatedByRealm() {
    return createdByRealm;
  }

  public void setCreatedByRealm(final String createdByRealm) {
    this.createdByRealm = createdByRealm;
  }
}
