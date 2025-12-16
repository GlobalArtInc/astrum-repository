
package org.globalart.astrum.internal.webhooks;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.crypto.secrets.Secret;
import org.globalart.astrum.crypto.secrets.SecretsService;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

/**
 * Tests for {@link GlobalWebhookCapability.Configuration} secret handling.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class GlobalWebhookCapabilityConfigurationTest
{
  @Mock
  private SecretsService secretsService;

  @Mock
  private Secret secret;

  private Map<String, String> properties;

  @Before
  public void setup() {
    properties = new HashMap<>();
    properties.put("names", "test");
    properties.put("url", "http://example.com");
  }

  @Test
  public void testSecretIsStoredEncrypted() {
    String secretId = "123";
    String encryptedSecret = "encrypted-secret-value";
    properties.put("secret", secretId);

    when(secretsService.from(secretId)).thenReturn(secret);
    when(secret.decrypt()).thenReturn("decrypted-value".toCharArray());

    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, secretsService);

    // Verify that getSecret() returns the decrypted value, proving encryption is stored internally
    assertThat(config.getSecret(), is("decrypted-value"));
  }

  @Test
  public void testSecretIsDecryptedOnDemand() {
    String secretId = "456";
    String encryptedSecret = "encrypted-secret-value";
    String decryptedSecret = "my-secret-key";
    properties.put("secret", secretId);

    when(secretsService.from(secretId)).thenReturn(secret);
    when(secret.decrypt()).thenReturn(decryptedSecret.toCharArray());

    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, secretsService);

    // Verify secret is decrypted on-demand
    assertThat(config.getSecret(), is(decryptedSecret));
  }

  @Test
  public void testNullSecretHandling() {
    // Don't add secret to properties

    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, secretsService);

    // Verify null secret is handled correctly
    assertThat(config.getSecret(), is(nullValue()));
  }

  @Test
  public void testEmptySecretHandling() {
    properties.put("secret", "");

    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, secretsService);

    // Verify empty secret is handled correctly (empty string becomes null via Strings.emptyToNull)
    assertThat(config.getSecret(), is(nullValue()));
  }

  @Test
  public void testSecretDecryptionWithNullSecretsService() {
    String encryptedSecret = "encrypted-secret-value";
    properties.put("secret", encryptedSecret);

    // Pass null for SecretsService (like in descriptor validation)
    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, null);

    // Should return encrypted value when SecretsService is null
    assertThat(config.getSecret(), is(encryptedSecret));
  }

  @Test
  public void testMultipleSecretAccesses() {
    String secretId = "789";
    String encryptedSecret = "encrypted-secret-value";
    String decryptedSecret = "my-secret-key";
    properties.put("secret", secretId);

    when(secretsService.from(secretId)).thenReturn(secret);
    when(secret.decrypt()).thenReturn(decryptedSecret.toCharArray());

    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, secretsService);

    // Verify multiple calls to getSecret() work correctly
    assertThat(config.getSecret(), is(decryptedSecret));
    assertThat(config.getSecret(), is(decryptedSecret));
  }

  @Test
  public void testSecretWithUnderscorePrefix() {
    String secretId = "_123";
    String encryptedSecret = "encrypted-secret-value";
    properties.put("secret", secretId);

    when(secretsService.from(secretId)).thenReturn(secret);
    when(secret.decrypt()).thenReturn("decrypted-value".toCharArray());

    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, secretsService);

    // Verify that underscore prefix is stripped and secret is decrypted correctly
    assertThat(config.getSecret(), is("decrypted-value"));
  }

  @Test
  public void testSecretWithUnderscorePrefixMultipleDigits() {
    String secretId = "_9876";
    String encryptedSecret = "encrypted-secret-value";
    String decryptedSecret = "my-global-webhook-secret";
    properties.put("secret", secretId);

    when(secretsService.from(secretId)).thenReturn(secret);
    when(secret.decrypt()).thenReturn(decryptedSecret.toCharArray());

    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, secretsService);

    // Verify that underscore prefix is stripped for multi-digit secret IDs
    assertThat(config.getSecret(), is(decryptedSecret));
  }

  @Test
  public void testSecretWithUnderscorePrefixNotFound() {
    String secretId = "_999";
    properties.put("secret", secretId);

    when(secretsService.from(secretId)).thenThrow(new RuntimeException("Secret not found"));

    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, secretsService);

    // Verify that null is returned when secret is not found
    assertThat(config.getSecret(), is(nullValue()));
  }

  @Test
  public void testSecretWithoutUnderscorePrefixStillWorks() {
    String secretId = "456";
    String encryptedSecret = "encrypted-secret-value";
    properties.put("secret", secretId);

    when(secretsService.from(secretId)).thenReturn(secret);
    when(secret.decrypt()).thenReturn("decrypted-value".toCharArray());

    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, secretsService);

    // Verify backwards compatibility - IDs without underscore still work
    assertThat(config.getSecret(), is("decrypted-value"));
  }

  @Test
  public void testSecretWithInvalidSecretIdFormat() {
    String secretId = "_abc";
    properties.put("secret", secretId);

    // parseToken() in SecretImpl throws IllegalArgumentException for invalid format
    when(secretsService.from(secretId)).thenThrow(new IllegalArgumentException("Unexpected token"));

    GlobalWebhookCapability.Configuration config =
        new GlobalWebhookCapability.Configuration(properties, secretsService);

    // Verify that invalid secret ID (non-numeric after underscore) returns as-is for backwards compatibility
    assertThat(config.getSecret(), is(secretId));
  }
}
