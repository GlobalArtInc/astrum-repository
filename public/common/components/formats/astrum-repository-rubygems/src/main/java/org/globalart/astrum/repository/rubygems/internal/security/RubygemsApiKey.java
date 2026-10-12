package org.globalart.astrum.repository.rubygems.internal.security;

import javax.annotation.Nullable;

import jakarta.servlet.http.HttpServletRequest;

import org.globalart.astrum.security.authc.apikey.ApiKeyExtractor;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.net.HttpHeaders.AUTHORIZATION;

@Component
@Qualifier(RubygemsApiKey.NAME)
public class RubygemsApiKey
    implements ApiKeyExtractor
{
  public static final String NAME = "RubyGemsApiKey";

  public static final String PREFIX = NAME + ".";

  @Nullable
  @Override
  public String extract(final HttpServletRequest request) {
    String header = request.getHeader(AUTHORIZATION);
    if (header == null) {
      return null;
    }
    String value = header.strip();
    if (value.startsWith("Bearer ")) {
      value = value.substring("Bearer ".length()).strip();
    }
    if (value.startsWith(PREFIX) && value.length() > PREFIX.length() && value.indexOf(' ') < 0) {
      return value.substring(PREFIX.length());
    }
    return null;
  }
}
