
package org.globalart.astrum.crypto.secrets;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.ContinuationAware;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonIgnoreType;

/**
 * <p>
 * Internal representation of a stored secret. This is not intended for consumption outside of the
 * {@link SecretsService}.
 * </p>
 *
 * <p>
 * Note: jackson annotations are present to prevent accidental exposure if included in an audit.
 * </p>
 */
@JsonIgnoreType
public class SecretData
    implements ContinuationAware
{
  private Integer id;

  private String keyId;

  @JsonIgnore
  private String secret;

  private String purpose;

  private String userId;

  /**
   * The internal identifier of a stored secret.
   */
  @Nullable
  public Integer getId() {
    return id;
  }

  /**
   * The identifier of the key used to encrypt the associated secret.
   */
  public String getKeyId() {
    return keyId;
  }

  /**
   * The provided purpose of the secret, e.g. LDAP
   * 
   * @return
   */
  public String getPurpose() {
    return purpose;
  }

  /**
   * The encrypted secret, see also {@link #getKeyId()}
   * 
   * @return
   */
  public String getSecret() {
    return secret;
  }

  /**
   * The user who originated the request involving the persistence of the secret
   */
  @Nullable
  public String getUserId() {
    return userId;
  }

  public void setId(@Nullable final Integer id) {
    this.id = id;
  }

  public void setKeyId(final String keyId) {
    this.keyId = keyId;
  }

  public void setPurpose(final String purpose) {
    this.purpose = purpose;
  }

  public void setSecret(final String secret) {
    this.secret = secret;
  }

  public void setUserId(@Nullable final String userId) {
    this.userId = userId;
  }

  @Override
  public String nextContinuationToken() {
    return Integer.toString(id);
  }
}
