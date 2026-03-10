
package org.globalart.astrum.supportzip;

import java.util.Map;
import java.util.HashMap;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import com.google.common.collect.ImmutableMap;
import org.junit.Test;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.not;
import static org.junit.Assert.assertNotNull;
import static org.junit.Assert.assertNull;
import static org.globalart.astrum.supportzip.PasswordSanitizing.REPLACEMENT;

/**
 * Tests for {@link PasswordSanitizing}
 */
public class PasswordSanitizingTest
{
  private final String PASSWORD = "admin123";

  private final Map<String, String> SENSITIVE_DATA = Stream.of(
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
      "bearerTokenId",
      "bearerToken",
      "yumSigning",
      "terraformSigning",
      "accountKey",
      "destinationInstancePassword",
      "ASTRUM_DATASTORE_ASTRUM_PASSWORD",
      "ASTRUM_SECURITY_INITIAL_PASSWORD",
      "astrum.datastore.astrum.password")
      .collect(Collectors.toMap(key -> key, key -> PASSWORD));

  @Test
  public void testSanitizingSensitiveData() {
    PasswordSanitizing<Map<String, String>> passwordSanitizing = new PasswordSanitizing<>();
    Map<String, String> sanitized = passwordSanitizing.transform(SENSITIVE_DATA);
    assertNotNull(sanitized);
    assertThat(sanitized.toString(), not(containsString(PASSWORD)));
    assertThat(sanitized.keySet(), is(SENSITIVE_DATA.keySet()));
    SENSITIVE_DATA.keySet().forEach(field -> assertThat(sanitized.get(field), is(REPLACEMENT)));
  }

  @Test
  public void testPlainDataIsNotChanged() {
    PasswordSanitizing<Map<String, String>> passwordSanitizing = new PasswordSanitizing<>();
    Map<String, String> sensitiveData = ImmutableMap.of(
        "name", "John",
        "lastName", "Doe",
        "password", PASSWORD);
    Map<String, String> sanitized = passwordSanitizing.transform(sensitiveData);
    assertNotNull(sanitized);
    Map<String, String> expected = ImmutableMap.of(
        "name", "John",
        "lastName", "Doe",
        "password", REPLACEMENT);
    assertThat(sanitized, is(expected));
  }

  @Test
  public void testTransformNullReturnsNull() {
    PasswordSanitizing<Map<String, String>> passwordSanitizing = new PasswordSanitizing<>();
    assertNull(passwordSanitizing.transform(null));
  }

  @Test
  public void testTransformReturnsNullWhenSerializationThrows() {
    PasswordSanitizing<Map<String, Object>> passwordSanitizing = new PasswordSanitizing<>();
    Map<String, Object> data = new HashMap<>();
    data.put("bad", new ThrowingBean());
    assertNull(passwordSanitizing.transform(data));
  }

  // Bean whose getter throws during serialization; Jackson wraps it in a JsonMappingException
  // (a subclass of IOException), which PasswordSanitizing#transform catches before returning null.
  public static class ThrowingBean
  {
    public String getName() {
      throw new RuntimeException("Simulated serialization failure");
    }
  }
}
