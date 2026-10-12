package org.globalart.astrum.repository.gitlfs.internal;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;

/**
 * Reads and writes the JSON documents of the Git LFS API.
 */
public final class GitLfsJson
{
  public static final String MEDIA_TYPE = "application/vnd.git-lfs+json";

  private static final ObjectMapper MAPPER = new ObjectMapper();

  private GitLfsJson() {
  }

  public static ObjectNode object() {
    return MAPPER.createObjectNode();
  }

  public static Response response(final int status, final Object body) {
    return new Response.Builder()
        .status(status < 400 ? Status.success(status) : Status.failure(status))
        .payload(new BytesPayload(bytes(body), MEDIA_TYPE))
        .build();
  }

  public static Response ok(final Object body) {
    return response(200, body);
  }

  public static Response error(final int status, final String message) {
    return response(status, Map.of("message", message == null ? "" : message));
  }

  /**
   * Reads the JSON object of a request body.
   *
   * @throws GitLfsException when the body is missing or is not a JSON object
   */
  public static JsonNode read(@Nullable final Payload payload) {
    if (payload == null) {
      throw GitLfsException.badRequest("Missing request body");
    }
    return readOptional(payload);
  }

  /**
   * Reads the JSON object of a request body that may be left out.
   *
   * @return the object, or an empty object when there is no body
   * @throws GitLfsException when the body is not a JSON object
   */
  public static JsonNode readOptional(@Nullable final Payload payload) {
    if (payload == null) {
      return object();
    }
    try (InputStream in = payload.openInputStream()) {
      JsonNode node = MAPPER.readTree(in);
      if (node == null || node.isMissingNode()) {
        return object();
      }
      if (!node.isObject()) {
        throw GitLfsException.unprocessable("Request body must be a JSON object");
      }
      return node;
    }
    catch (IOException e) {
      throw GitLfsException.badRequest("Invalid JSON: " + e.getMessage());
    }
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
  public static String text(@Nullable final JsonNode node, final String field) {
    JsonNode value = node == null ? null : node.get(field);
    return value != null && value.isTextual() ? value.asText() : null;
  }
}
