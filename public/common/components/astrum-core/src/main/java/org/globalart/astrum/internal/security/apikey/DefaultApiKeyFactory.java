
package org.globalart.astrum.internal.security.apikey;

import java.util.UUID;

import org.globalart.astrum.security.authc.apikey.ApiKeyFactory;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;

/**
 * Default {@link ApiKeyFactory} that creates random UUID v4 API keys.
 * <p>
 * Uses {@link java.security.SecureRandom} internally through {@link UUID#randomUUID()}.
 * This provides 122 bits of entropy from a cryptographically secure random source.
 */
@Primary
@Component
@Qualifier("default")
public class DefaultApiKeyFactory
    implements ApiKeyFactory
{
  @Override
  public char[] makeApiKey() {
    final String apiKey = UUID.randomUUID().toString();
    return apiKey.toCharArray();
  }
}
