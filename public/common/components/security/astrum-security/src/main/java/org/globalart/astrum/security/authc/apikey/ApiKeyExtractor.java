
package org.globalart.astrum.security.authc.apikey;

import javax.annotation.Nullable;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.security.authc.AstrumApiKeyAuthenticationToken;

/**
 * API-Key extractor component. It extracts the format specific API-Key if possible. Can use multiple headers to
 * extract key as needed. Extracted key will end up as {@link AstrumApiKeyAuthenticationToken} credential and with this
 * named component' name as token principal. This implies that a realm implementation must exist that handles {@link
 * AstrumApiKeyAuthenticationToken}s that has {@link AstrumApiKeyAuthenticationToken#getPrincipal()} equal as name of
 * named component implementing this interface.
 */
public interface ApiKeyExtractor
{
  /**
   * Attempts to extract API key as string, whatever part (or parts) of the {@link HttpServletRequest}
   * it needs and returns the extracted key, or returns {@code null}.
   */
  @Nullable
  String extract(HttpServletRequest request);
}
