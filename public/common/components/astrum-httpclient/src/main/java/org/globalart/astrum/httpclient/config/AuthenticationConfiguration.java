
package org.globalart.astrum.httpclient.config;

import java.util.List;
import java.util.Map;

import org.globalart.astrum.crypto.secrets.Secret;

import com.fasterxml.jackson.annotation.JsonIgnore;
import com.fasterxml.jackson.annotation.JsonSubTypes;
import com.fasterxml.jackson.annotation.JsonSubTypes.Type;
import com.fasterxml.jackson.annotation.JsonTypeInfo;
import com.google.common.collect.ImmutableMap;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Authentication configuration.
 *
 * @since 3.0
 */
@JsonTypeInfo(
    use = JsonTypeInfo.Id.NAME,
    property = "type")
@JsonSubTypes({
    @Type(value = BearerTokenAuthenticationConfiguration.class, name = BearerTokenAuthenticationConfiguration.TYPE),
    @Type(value = NtlmAuthenticationConfiguration.class, name = NtlmAuthenticationConfiguration.TYPE),
    @Type(value = UsernameAuthenticationConfiguration.class, name = UsernameAuthenticationConfiguration.TYPE)
})
public abstract class AuthenticationConfiguration
    implements Cloneable
{
  /**
   * Mapping of type-name to type-class used by deserializers. If you add new type of auth config, this map needs
   * to be updated as well.
   */
  public static final Map<String, Class<? extends AuthenticationConfiguration>> TYPES = ImmutableMap.of(
      UsernameAuthenticationConfiguration.TYPE, UsernameAuthenticationConfiguration.class,
      NtlmAuthenticationConfiguration.TYPE, NtlmAuthenticationConfiguration.class,
      BearerTokenAuthenticationConfiguration.TYPE, BearerTokenAuthenticationConfiguration.class);

  public static final String AUTHENTICATION_CONFIGURATION = "authentication-config";

  private final String type;

  private boolean preemptive;

  public AuthenticationConfiguration(final String type) {
    this.type = checkNotNull(type);
  }

  public String getType() {
    return type;
  }

  public boolean isPreemptive() {
    return preemptive;
  }

  public void setPreemptive(final boolean preemptive) {
    this.preemptive = preemptive;
  }

  public abstract Secret getSecret();

  /**
   * Returns the field names that contain secret values in the JSON representation.
   * Used for placeholder detection when validating authentication changes.
   */
  @JsonIgnore
  public abstract List<String> getSecretFieldNames();

  public AuthenticationConfiguration copy() {
    try {
      return (AuthenticationConfiguration) clone();
    }
    catch (CloneNotSupportedException e) {
      throw new RuntimeException(e);
    }
  }
}
