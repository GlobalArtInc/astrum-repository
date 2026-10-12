package org.globalart.astrum.repository.terraform.internal;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.util.List;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.databind.node.ObjectNode;

import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

/**
 * JSON documents of the Terraform protocols and the responses that carry them.
 */
public final class TerraformResponses
{
  public static final String X_TERRAFORM_GET = "X-Terraform-Get";

  public static final ObjectMapper MAPPER = new ObjectMapper().disable(SerializationFeature.FAIL_ON_EMPTY_BEANS);

  private TerraformResponses() {
  }

  public static Response json(final JsonNode body) {
    return json(200, body);
  }

  public static Response json(final int status, final JsonNode body) {
    return new Response.Builder()
        .status(status < 300 ? Status.success(status) : Status.failure(status))
        .payload(new BytesPayload(bytes(body), APPLICATION_JSON))
        .build();
  }

  /**
   * An error in the shape registries use, {@code {"errors": ["..."]}}; Terraform prints it with the status.
   */
  public static Response error(final int status, final String message) {
    ObjectNode body = object();
    body.putArray("errors").add(message);
    return new Response.Builder()
        .status(Status.failure(status, message))
        .payload(new BytesPayload(bytes(body), APPLICATION_JSON))
        .build();
  }

  /**
   * The answer of a module download endpoint: no content, the package location in {@code X-Terraform-Get}.
   */
  public static Response location(final String location) {
    return new Response.Builder()
        .status(Status.success(204))
        .header(X_TERRAFORM_GET, location)
        .build();
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

  public static JsonNode read(final byte[] bytes) throws IOException {
    return MAPPER.readTree(bytes);
  }

  @Nullable
  public static JsonNode read(@Nullable final Payload payload) throws IOException {
    if (payload == null) {
      return null;
    }
    try (InputStream in = payload.openInputStream()) {
      return MAPPER.readTree(in);
    }
  }

  /**
   * The body of a successful response, or {@code null} when the response failed or had no body.
   */
  @Nullable
  public static byte[] body(final Response response) throws IOException {
    Payload payload = response.getPayload();
    if (!response.getStatus().isSuccessful() || payload == null) {
      return null;
    }
    try (Payload closeable = payload; InputStream in = closeable.openInputStream()) {
      return in.readAllBytes();
    }
  }

  public static ObjectNode discovery(final String modulesUrl, final String providersUrl) {
    ObjectNode document = object();
    document.put("modules.v1", modulesUrl);
    document.put("providers.v1", providersUrl);
    return document;
  }

  public static ObjectNode moduleVersions(final List<String> versions) {
    ObjectNode document = object();
    var entries = document.putArray("modules").addObject().putArray("versions");
    versions.forEach(version -> entries.addObject().put("version", version));
    return document;
  }

  public static ObjectNode mirrorIndex(final List<String> versions) {
    ObjectNode document = object();
    ObjectNode entries = document.putObject("versions");
    versions.forEach(version -> entries.putObject(version));
    return document;
  }
}
