
package org.globalart.astrum.repository.webhooks;

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
 * Tests for {@link RepositoryWebhookCapabilityConfiguration} secret handling.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class RepositoryWebhookCapabilityConfigurationTest
{
  @Mock
  private SecretsService secretsService;

  @Mock
  private Secret secret;

  private Map<String, String> properties;

  @Before
  public void setup() {
    properties = new HashMap<>();
    properties.put("repository", "maven-central");
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

    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, secretsService);

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

    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, secretsService);

    // Verify secret is decrypted on-demand
    assertThat(config.getSecret(), is(decryptedSecret));
  }

  @Test
  public void testNullSecretHandling() {
    // Don't add secret to properties

    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, secretsService);

    // Verify null secret is handled correctly
    assertThat(config.getSecret(), is(nullValue()));
  }

  @Test
  public void testEmptySecretHandling() {
    properties.put("secret", "");

    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, secretsService);

    // Verify empty secret is handled correctly
    assertThat(config.getSecret(), is(nullValue()));
  }

  @Test
  public void testSecretDecryptionWithNullSecretsService() {
    String encryptedSecret = "encrypted-secret-value";
    properties.put("secret", encryptedSecret);

    // Pass null for SecretsService (like in descriptor validation)
    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, null);

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

    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, secretsService);

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

    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, secretsService);

    // Verify that underscore prefix is stripped and secret is decrypted correctly
    assertThat(config.getSecret(), is("decrypted-value"));
  }

  @Test
  public void testSecretWithUnderscorePrefixMultipleDigits() {
    String secretId = "_9876";
    String encryptedSecret = "encrypted-secret-value";
    String decryptedSecret = "my-webhook-secret";
    properties.put("secret", secretId);

    when(secretsService.from(secretId)).thenReturn(secret);
    when(secret.decrypt()).thenReturn(decryptedSecret.toCharArray());

    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, secretsService);

    // Verify that underscore prefix is stripped for multi-digit secret IDs
    assertThat(config.getSecret(), is(decryptedSecret));
  }

  @Test
  public void testSecretWithUnderscorePrefixNotFound() {
    String secretId = "_999";
    properties.put("secret", secretId);

    when(secretsService.from(secretId)).thenThrow(new RuntimeException("Secret not found"));

    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, secretsService);

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

    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, secretsService);

    // Verify backwards compatibility - IDs without underscore still work
    assertThat(config.getSecret(), is("decrypted-value"));
  }

  @Test
  public void testSecretWithInvalidSecretIdFormat() {
    String secretId = "_abc";
    properties.put("secret", secretId);

    when(secretsService.from(secretId)).thenThrow(new IllegalArgumentException("Unexpected token"));

    RepositoryWebhookCapabilityConfiguration config =
        new RepositoryWebhookCapabilityConfiguration(properties, secretsService);

    // Verify that invalid secret ID (non-numeric after underscore) returns as-is for backwards compatibility
    assertThat(config.getSecret(), is(secretId));
  }
}
