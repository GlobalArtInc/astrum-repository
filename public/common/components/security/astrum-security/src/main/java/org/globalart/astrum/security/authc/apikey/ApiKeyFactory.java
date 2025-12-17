
package org.globalart.astrum.security.authc.apikey;

/**
 * API Key Factory that creates API Keys. If some specific content or format needed, implement one as component and
 * use same name as your {@link ApiKeyExtractor} component has.
 */
public interface ApiKeyFactory
{
  /**
   * Creates a domain specific API Key, never {@code null}.
   */
  char[] makeApiKey();
}
