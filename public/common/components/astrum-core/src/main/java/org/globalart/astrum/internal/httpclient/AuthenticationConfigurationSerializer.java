
package org.globalart.astrum.internal.httpclient;

import java.io.IOException;

import org.globalart.astrum.httpclient.config.AuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.BearerTokenAuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.NtlmAuthenticationConfiguration;
import org.globalart.astrum.httpclient.config.UsernameAuthenticationConfiguration;
import org.globalart.astrum.kv.KeyValueStore;

import com.fasterxml.jackson.core.JsonGenerationException;
import com.fasterxml.jackson.core.JsonGenerator;
import com.fasterxml.jackson.databind.SerializerProvider;
import com.fasterxml.jackson.databind.jsontype.TypeSerializer;
import com.fasterxml.jackson.databind.ser.std.StdSerializer;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.manager.internal.HttpAuthenticationSecretEncoder.BEARER_TOKEN_MIGRATION_STARTED;

/**
 * {@link AuthenticationConfiguration} serializer.
 * <p>
 * Encrypts sensitive data.
 *
 * @since 3.0
 */
public class AuthenticationConfigurationSerializer
    extends StdSerializer<AuthenticationConfiguration>
{
  private final KeyValueStore keyValueStore;

  public AuthenticationConfigurationSerializer(final KeyValueStore keyValueStore) {
    super(AuthenticationConfiguration.class);
    this.keyValueStore = checkNotNull(keyValueStore);
  }

  private boolean isMigrationStarted() {
    return keyValueStore.getBoolean(BEARER_TOKEN_MIGRATION_STARTED).orElse(false);
  }

  @Override
  public void serialize(
      final AuthenticationConfiguration value,
      final JsonGenerator jgen,
      final SerializerProvider provider) throws IOException
  {
    serialize(value, jgen);
    jgen.writeEndObject();
  }

  @Override
  public void serializeWithType(
      final AuthenticationConfiguration value,
      final JsonGenerator jgen,
      final SerializerProvider provider,
      final TypeSerializer typeSer) throws IOException
  {
    serialize(value, jgen);
    if (value instanceof UsernameAuthenticationConfiguration) {
      jgen.writeStringField(typeSer.getPropertyName(), UsernameAuthenticationConfiguration.TYPE);
    }
    else if (value instanceof NtlmAuthenticationConfiguration) {
      jgen.writeStringField(typeSer.getPropertyName(), NtlmAuthenticationConfiguration.TYPE);
    }
    else if (value instanceof BearerTokenAuthenticationConfiguration) {
      jgen.writeStringField(typeSer.getPropertyName(), BearerTokenAuthenticationConfiguration.TYPE);
    }
    else {
      // be foolproof, if new type added but this class is not updated
      throw new JsonGenerationException("Unsupported type:" + value.getClass().getName(), jgen);
    }
    jgen.writeEndObject();
  }

  private void serialize(final AuthenticationConfiguration value, final JsonGenerator jgen) throws IOException {
    jgen.writeStartObject();
    jgen.writeStringField("type", value.getType());
    jgen.writeBooleanField("preemptive", value.isPreemptive());
    if (value instanceof UsernameAuthenticationConfiguration) {
      UsernameAuthenticationConfiguration upc = (UsernameAuthenticationConfiguration) value;
      jgen.writeStringField("username", upc.getUsername());
      if (upc.getPassword() != null) {
        jgen.writeStringField("password", upc.getPassword().getId());
      }
    }
    else if (value instanceof NtlmAuthenticationConfiguration) {
      NtlmAuthenticationConfiguration ntc = (NtlmAuthenticationConfiguration) value;
      jgen.writeStringField("username", ntc.getUsername());
      if (ntc.getPassword() != null) {
        jgen.writeStringField("password", ntc.getPassword().getId());
      }
      jgen.writeStringField("domain", ntc.getDomain());
      jgen.writeStringField("host", ntc.getHost());
    }
    else if (value instanceof BearerTokenAuthenticationConfiguration) {
      BearerTokenAuthenticationConfiguration btac = (BearerTokenAuthenticationConfiguration) value;
      if (btac.getBearerToken() != null) {
        if (isMigrationStarted()) {
          // Post-migration: write as bearerTokenId (encrypted secret reference)
          jgen.writeStringField("bearerTokenId", btac.getBearerToken().getId());
        }
        else {
          // Pre-migration: write as bearerToken (plain text)
          jgen.writeStringField("bearerToken", btac.getBearerToken().getId());
        }
      }
    }
    else {
      // be foolproof, if new type added but this class is not updated
      throw new JsonGenerationException("Unsupported type:" + value.getClass().getName());
    }
  }
}
