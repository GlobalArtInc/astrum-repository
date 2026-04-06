package org.globalart.astrum.repository.nuget.internal.recipe;

import java.nio.charset.StandardCharsets;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetV3Documents;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.matchers.RegexMatcher;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.payloads.StringPayload;

public final class NugetResponses
{
  private NugetResponses() {
  }

  public static Response json(final String json) {
    return HttpResponses.ok(new StringPayload(json, StandardCharsets.UTF_8, NugetV3Documents.JSON));
  }

  public static Response xml(final String xml, final String contentType) {
    return HttpResponses.ok(new StringPayload(xml, StandardCharsets.UTF_8, contentType));
  }

  public static Response text(final String text) {
    return HttpResponses.ok(new StringPayload(text, StandardCharsets.UTF_8, "text/plain"));
  }

  public static Response status(final int code, @Nullable final String message) {
    return new Response.Builder().status(new Status(code >= 200 && code < 300, code, message)).build();
  }

  public static Response noContent() {
    return status(HttpStatus.NO_CONTENT, null);
  }

  public static Map<String, String> tokens(final Context context) {
    return context.getAttributes().require(TokenMatcher.State.class).getTokens();
  }

  public static String group(final Context context, final int index) {
    return context.getAttributes().require(RegexMatcher.State.class).getMatchResult().group(index);
  }

  public static String repositoryUrl(final Context context) {
    return context.getRepository().getUrl();
  }

  public static String unquote(@Nullable final String value) {
    if (value == null) {
      return null;
    }
    String trimmed = value.trim();
    if (trimmed.length() >= 2 && trimmed.startsWith("'") && trimmed.endsWith("'")) {
      return trimmed.substring(1, trimmed.length() - 1).replace("''", "'");
    }
    return trimmed;
  }
}
