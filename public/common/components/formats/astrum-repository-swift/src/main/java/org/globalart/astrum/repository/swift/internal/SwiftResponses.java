package org.globalart.astrum.repository.swift.internal;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.LinkedHashMap;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.google.common.hash.Hashing;

public final class SwiftResponses
{
  public static final ObjectMapper MAPPER = new ObjectMapper();

  public static final String API_VERSION = "1";

  public static final String CONTENT_VERSION = "Content-Version";

  public static final String CONTENT_LANGUAGE = "Content-Language";

  public static final String CONTENT_DISPOSITION = "Content-Disposition";

  public static final String CACHE_CONTROL = "Cache-Control";

  public static final String LINK = "Link";

  public static final String LOCATION = "Location";

  public static final String DIGEST = "Digest";

  public static final String SIGNATURE_FORMAT = "X-Swift-Package-Signature-Format";

  public static final String SIGNATURE = "X-Swift-Package-Signature";

  public static final String APPLICATION_JSON = "application/json";

  public static final String PROBLEM_JSON = "application/problem+json";

  public static final String TEXT_X_SWIFT = "text/x-swift";

  public static final String APPLICATION_ZIP = "application/zip";

  public static final String IMMUTABLE = "public, immutable";

  private SwiftResponses() {
  }

  public static Response problem(final int status, final String detail) {
    Map<String, Object> body = new LinkedHashMap<>();
    body.put("status", status);
    body.put("title", title(status));
    body.put("detail", detail == null ? title(status) : detail);
    return new Response.Builder()
        .status(Status.failure(status, title(status)))
        .header(CONTENT_VERSION, API_VERSION)
        .header(CONTENT_LANGUAGE, "en")
        .payload(new BytesPayload(bytes(body), PROBLEM_JSON))
        .build();
  }

  public static Response json(final JsonNode body) {
    return json(body, 200);
  }

  public static Response json(final JsonNode body, final int status) {
    return new Response.Builder()
        .status(Status.success(status))
        .header(CONTENT_VERSION, API_VERSION)
        .payload(content(bytes(body), APPLICATION_JSON))
        .build();
  }

  public static Response empty(final int status) {
    return new Response.Builder()
        .status(Status.success(status))
        .header(CONTENT_VERSION, API_VERSION)
        .build();
  }

  public static Response seeOther(final String location) {
    return new Response.Builder()
        .status(Status.success(303))
        .header(CONTENT_VERSION, API_VERSION)
        .header(LOCATION, location)
        .build();
  }

  public static Content content(final byte[] bytes, final String contentType) {
    Content content = new Content(new BytesPayload(bytes, contentType));
    content.getAttributes().set(Content.CONTENT_ETAG, Hashing.sha256().hashBytes(bytes).toString().substring(0, 40));
    return content;
  }

  public static String attachment(final String filename) {
    return "attachment; filename=\"" + filename + "\"";
  }

  public static ObjectNode object() {
    return MAPPER.createObjectNode();
  }

  public static byte[] bytes(final Object body) {
    try {
      return MAPPER.writeValueAsBytes(body);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  @Nullable
  public static JsonNode readQuietly(@Nullable final byte[] bytes) {
    if (bytes == null) {
      return null;
    }
    try {
      return MAPPER.readTree(bytes);
    }
    catch (IOException e) {
      return null;
    }
  }

  public static String title(final int status) {
    return switch (status) {
      case 400 -> "Bad Request";
      case 401 -> "Unauthorized";
      case 403 -> "Forbidden";
      case 404 -> "Not Found";
      case 405 -> "Method Not Allowed";
      case 409 -> "Conflict";
      case 413 -> "Payload Too Large";
      case 415 -> "Unsupported Media Type";
      case 422 -> "Unprocessable Entity";
      case 502 -> "Bad Gateway";
      case 503 -> "Service Unavailable";
      default -> status >= 500 ? "Server Error" : "Error";
    };
  }
}
