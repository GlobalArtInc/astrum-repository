
package org.globalart.astrum.supportzip;

import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.List;
import java.util.Map;
import javax.annotation.Nullable;
import org.globalart.astrum.common.io.SanitizingJsonOutputStream;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.ObjectReader;
import com.fasterxml.jackson.databind.ObjectWriter;
import com.google.common.collect.ImmutableList;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

/**
 * Should be used to hide sensitive data like password in the {@link Map}.
 *
 * @since 3.29
 */
@Component
public class PasswordSanitizing<T extends Map<String, ?>>
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  public static final List<String> SENSITIVE_FIELD_NAMES = ImmutableList.of(
      "applicationPassword",
      "password",
      "systemPassword",
      "keyStorePassword",
      "secret",
      "accessKeyId",
      "secretAccessKey",
      "sessionToken",
      "auth_account_key",
      "aptSigning",
      "alpineSigning",
      "bearerToken",
      "bearerTokenId",
      "yumSigning",
      "terraformSigning",
      "accountKey",
      "destinationInstancePassword",
      "ASTRUM_DATASTORE_ASTRUM_PASSWORD",
      "ASTRUM_SECURITY_INITIAL_PASSWORD",
      "astrum.datastore.astrum.password");

  public static final String REPLACEMENT = "**REDACTED**";

  private final TypeReference<T> typeReference = new TypeReference<T>()
  {
  };

  private final ObjectWriter attributesJsonWriter = new ObjectMapper().writerFor(typeReference);

  private final ObjectReader attributesJsonReader = new ObjectMapper().readerFor(typeReference);

  /**
   * Replace sensitive data by {@code REPLACEMENT}.
   *
   * @param value sensitive data.
   * @return transformed data.
   */
  @Nullable
  public T transform(T value) {
    try (ByteArrayOutputStream obfuscatedAttrs = new ByteArrayOutputStream()) {
      try (SanitizingJsonOutputStream sanitizer = new SanitizingJsonOutputStream(
          obfuscatedAttrs,
          SENSITIVE_FIELD_NAMES,
          REPLACEMENT)) {
        sanitizer.write(attributesJsonWriter.writeValueAsBytes(value));
      }

      Object result = attributesJsonReader.readValue(obfuscatedAttrs.toByteArray());
      return result != null ? (T) result : null;
    }
    catch (IOException e) {
      log.error("Error obfuscating attributes", e);
    }

    return null;
  }
}
