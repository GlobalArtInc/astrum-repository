package org.globalart.astrum.repository.swift.internal;

import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

public final class ApiVersions
{
  private static final Pattern REGISTRY_MEDIA_TYPE = Pattern.compile(
      "application/vnd\\.swift\\.registry(?:\\.v([^+;,\\s]*))?(?:\\+([^;,\\s]*))?", Pattern.CASE_INSENSITIVE);

  private static final Pattern DIGITS = Pattern.compile("\\d+");

  private ApiVersions() {
  }

  public static Optional<SwiftProblem> check(@Nullable final String accept) {
    if (accept == null) {
      return Optional.empty();
    }
    Matcher matcher = REGISTRY_MEDIA_TYPE.matcher(accept);
    if (!matcher.find()) {
      return Optional.empty();
    }
    String version = matcher.group(1);
    if (version == null || SwiftResponses.API_VERSION.equals(version)) {
      return Optional.empty();
    }
    if (DIGITS.matcher(version).matches()) {
      return Optional.of(SwiftProblem.unsupportedMediaType("unsupported API version"));
    }
    return Optional.of(SwiftProblem.badRequest("invalid API version"));
  }
}
