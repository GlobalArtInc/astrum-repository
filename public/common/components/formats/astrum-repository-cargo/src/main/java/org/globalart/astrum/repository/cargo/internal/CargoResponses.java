package org.globalart.astrum.repository.cargo.internal;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.node.ObjectNode;
import com.google.common.hash.Hashing;

import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

public final class CargoResponses
{
  public static final ObjectMapper MAPPER = new ObjectMapper();

  private CargoResponses() {
  }

  public static Response json(final Object body) {
    return new Response.Builder()
        .status(Status.success(200))
        .payload(new BytesPayload(bytes(body), APPLICATION_JSON))
        .build();
  }

  public static Response ok() {
    return json(Map.of("ok", true));
  }

  public static Response error(final int status, final String detail) {
    byte[] body = bytes(Map.of("errors", List.of(Map.of("detail", detail == null ? "" : detail))));
    return new Response.Builder()
        .status(Status.failure(status, detail))
        .payload(new BytesPayload(body, APPLICATION_JSON))
        .build();
  }

  public static Content content(final byte[] bytes, final String contentType) {
    Content content = new Content(new BytesPayload(bytes, contentType));
    content.getAttributes().set(Content.CONTENT_ETAG, etag(bytes));
    return content;
  }

  public static String etag(final byte[] bytes) {
    return Hashing.sha256().hashBytes(bytes).toString().substring(0, 40);
  }

  public static byte[] bytes(final Object body) {
    try {
      return MAPPER.writeValueAsBytes(body);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  public static ObjectNode object() {
    return MAPPER.createObjectNode();
  }

  public static JsonNode read(final byte[] bytes) {
    try {
      return MAPPER.readTree(bytes);
    }
    catch (IOException e) {
      throw CargoException.badRequest("Invalid JSON: " + e.getMessage());
    }
  }
}
