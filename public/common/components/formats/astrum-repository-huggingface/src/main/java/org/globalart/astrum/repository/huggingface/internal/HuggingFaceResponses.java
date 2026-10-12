package org.globalart.astrum.repository.huggingface.internal;

import java.io.IOException;
import java.io.UncheckedIOException;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.ETagHeaderUtils;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.google.common.net.HttpHeaders;

import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

/**
 * Responses with the headers Hub clients read.
 */
public final class HuggingFaceResponses
{
  public static final String X_REPO_COMMIT = "X-Repo-Commit";

  public static final String X_LINKED_ETAG = "X-Linked-Etag";

  public static final String X_LINKED_SIZE = "X-Linked-Size";

  public static final String X_ERROR_CODE = "X-Error-Code";

  public static final String X_ERROR_MESSAGE = "X-Error-Message";

  private static final ObjectMapper MAPPER = new ObjectMapper();

  private HuggingFaceResponses() {
  }

  public static Response json(final byte[] body) {
    return HttpResponses.ok(new BytesPayload(body, APPLICATION_JSON));
  }

  public static Response json(final JsonNode node) {
    try {
      return json(MAPPER.writeValueAsBytes(node));
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  public static Response error(final HuggingFaceException exception) {
    String message = headerValue(exception.getMessage());
    ObjectNode body = MAPPER.createObjectNode().put("error", message);
    Response.Builder builder = new Response.Builder()
        .status(Status.failure(exception.getStatus(), message))
        .header(X_ERROR_MESSAGE, message);
    if (exception.getErrorCode() != null) {
      builder.header(X_ERROR_CODE, exception.getErrorCode());
    }
    if (exception.getCommit() != null) {
      builder.header(X_REPO_COMMIT, exception.getCommit());
    }
    try {
      builder.payload(new BytesPayload(MAPPER.writeValueAsBytes(body), APPLICATION_JSON));
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
    return builder.build();
  }

  /**
   * A file answer: the commit it was served from and the ETag and size Hub clients key their cache by.
   */
  public static Response file(final Payload payload, final String commit, @Nullable final String etag, final long size) {
    Response.Builder builder = new Response.Builder()
        .status(Status.success(200))
        .payload(payload)
        .header(X_REPO_COMMIT, commit)
        .header(HttpHeaders.ACCEPT_RANGES, "bytes");
    if (etag != null) {
      builder.header(HttpHeaders.ETAG, ETagHeaderUtils.quote(etag));
      builder.header(X_LINKED_ETAG, ETagHeaderUtils.quote(etag));
    }
    if (size >= 0) {
      builder.header(X_LINKED_SIZE, Long.toString(size));
    }
    return builder.build();
  }

  static String headerValue(@Nullable final String value) {
    if (value == null || value.isBlank()) {
      return "Not found";
    }
    StringBuilder result = new StringBuilder(value.length());
    for (char c : value.toCharArray()) {
      result.append(c >= 0x20 && c < 0x7f ? c : ' ');
    }
    String text = result.toString().trim();
    return text.length() > 500 ? text.substring(0, 500) : text;
  }
}
