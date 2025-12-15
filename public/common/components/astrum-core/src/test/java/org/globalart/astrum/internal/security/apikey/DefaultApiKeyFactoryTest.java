
package org.globalart.astrum.internal.security.apikey;

import java.util.UUID;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Tests for {@link DefaultApiKeyFactory}.
 */
public class DefaultApiKeyFactoryTest
{
  @Test
  public void makeApiKey_returnsValidUUID() {
    DefaultApiKeyFactory factory = new DefaultApiKeyFactory();

    char[] apiKey = factory.makeApiKey();
    String apiKeyString = new String(apiKey);

    // Verify it's a valid UUID
    UUID uuid = UUID.fromString(apiKeyString);

    assertThat("Generated key must be a valid UUID", uuid, is(notNullValue()));
    assertThat("API key must be 36 characters", apiKeyString.length(), is(36));
  }
}
