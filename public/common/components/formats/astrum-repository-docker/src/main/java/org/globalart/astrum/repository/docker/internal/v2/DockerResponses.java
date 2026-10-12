package org.globalart.astrum.repository.docker.internal.v2;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.Status;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.databind.ObjectMapper;

public final class DockerResponses
{
  public static final String API_VERSION_HEADER = "Docker-Distribution-API-Version";

  public static final String API_VERSION = "registry/2.0";

  public static final String CONTENT_DIGEST_HEADER = "Docker-Content-Digest";

  public static final String UPLOAD_UUID_HEADER = "Docker-Upload-UUID";

  public static final String OCI_SUBJECT_HEADER = "OCI-Subject";

  public static final String OCI_TAG_HEADER = "OCI-Tag";

  public static final String WWW_AUTHENTICATE = "WWW-Authenticate";

  public static final String JSON_CONTENT_TYPE = "application/json; charset=utf-8";

  public static final ObjectMapper MAPPER = new ObjectMapper();

  public static final String ERROR_ATTRIBUTE = DockerResponses.class.getName() + ".error";

  private DockerResponses() {
  }

  public static Response error(final DockerError error) {
    return error(error, error.message(), null);
  }

  public static Response error(final DockerError error, final String message) {
    return error(error, message, null);
  }

  public static Response error(final DockerError error, final String message, @Nullable final Object detail) {
    return error(error.status(), error, message, detail);
  }

  public static Response error(
      final int status,
      final DockerError error,
      final String message,
      @Nullable final Object detail)
  {
    Map<String, Object> entry = new LinkedHashMap<>();
    entry.put("code", error.name());
    entry.put("message", message);
    if (detail != null) {
      entry.put("detail", detail);
    }
    Map<String, Object> body = Map.of("errors", List.of(entry));
    return new Response.Builder()
        .status(Status.failure(status))
        .header(API_VERSION_HEADER, API_VERSION)
        .attribute(ERROR_ATTRIBUTE, error)
        .payload(json(body))
        .build();
  }

  public static Response.Builder success(final int status) {
    return new Response.Builder()
        .status(Status.success(status))
        .header(API_VERSION_HEADER, API_VERSION);
  }

  public static Response jsonResponse(final int status, final Object body) {
    return success(status).payload(json(body)).build();
  }

  public static BytesPayload json(final Object body) {
    return json(body, JSON_CONTENT_TYPE);
  }

  public static BytesPayload json(final Object body, final String contentType) {
    try {
      return new BytesPayload(MAPPER.writeValueAsBytes(body), contentType);
    }
    catch (JsonProcessingException e) {
      throw new IllegalStateException(e);
    }
  }

  public static Response empty(final int status) {
    return success(status).header("Content-Length", "0").build();
  }
}
