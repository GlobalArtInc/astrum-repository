
package org.globalart.astrum.internal.security.apikey.store;

import java.time.OffsetDateTime;

import org.globalart.astrum.internal.security.apikey.ApiKeyInternal;

import org.apache.shiro.subject.PrincipalCollection;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * {@link ApiKeyInternal} data.
 *
 * @since 3.21
 */
public class ApiKeyData
    implements ApiKeyInternal
{
  private String domain;

  private PrincipalCollection principals;

  private ApiKeyToken token;

  private OffsetDateTime created;

  // Public no-arg constructor so tests in other modules can build instances via the public setters without
  // a same-package subclass (avoids a split-package across modules).
  public ApiKeyData() {
  }

  ApiKeyData(
      final String domain,
      final PrincipalCollection principals,
      final ApiKeyToken token,
      final OffsetDateTime created)
  {
    this.domain = checkNotNull(domain);
    this.principals = checkNotNull(principals);
    this.token = checkNotNull(token);
    this.created = created;
  }

  @Override
  public void setDomain(final String domain) {
    this.domain = domain;
  }

  @Override
  public void setPrincipals(final PrincipalCollection principals) {
    this.principals = principals;
  }

  public void setToken(final ApiKeyToken token) {
    this.token = token;
  }

  public void setApiKey(final char[] chars) {
    this.token = new ApiKeyToken(chars);
  }

  @Override
  public String getDomain() {
    return domain;
  }

  @Override
  public PrincipalCollection getPrincipals() {
    return principals;
  }

  public ApiKeyToken getToken() {
    return token;
  }

  @Override
  public char[] getApiKey() {
    return token.getChars();
  }

  @Override
  public OffsetDateTime getCreated() {
    return created;
  }

  @Override
  public void setCreated(final OffsetDateTime created) {
    this.created = created;
  }
}
