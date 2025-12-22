
package org.globalart.astrum.security.token;

import java.util.List;

import javax.annotation.Nullable;
import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.security.authc.apikey.ApiKeyExtractor;

import com.google.common.base.Splitter;
import com.google.common.collect.Lists;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.net.HttpHeaders.AUTHORIZATION;

/**
 * Extracts a token from a HttpServletRequest
 *
 * @since 3.6
 */
public class BearerToken
    implements ApiKeyExtractor
{
  private final String format;

  public BearerToken(final String format) {
    this.format = checkNotNull(format);
  }

  @Nullable
  @Override
  public String extract(final HttpServletRequest request) {
    final String headerValue = request.getHeader(AUTHORIZATION);
    if (headerValue != null && headerValue.startsWith("Bearer ")) {
      List<String> parts = Lists.newArrayList(Splitter.on(' ').split(headerValue));
      if (parts.size() == 2 && "Bearer".equals(parts.get(0)) && matchesFormat(parts)) {
        return parts.get(1).replaceAll(format + ".", "");
      }
    }
    return null;
  }

  protected boolean matchesFormat(final List<String> parts) {
    return parts.get(1).startsWith(format);
  }
}
