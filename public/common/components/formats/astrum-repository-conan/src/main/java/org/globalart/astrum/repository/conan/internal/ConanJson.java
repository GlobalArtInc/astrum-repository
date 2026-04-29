package org.globalart.astrum.repository.conan.internal;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.fasterxml.jackson.core.JsonProcessingException;
import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;

import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

public final class ConanJson
{
  public static final ObjectMapper MAPPER = new ObjectMapper().enable(SerializationFeature.INDENT_OUTPUT);

  private static final TypeReference<LinkedHashMap<String, Object>> MAP = new TypeReference<>()
  {
  };

  private ConanJson() {
  }

  public static byte[] bytes(final Object value) {
    try {
      return MAPPER.writeValueAsBytes(value);
    }
    catch (JsonProcessingException e) {
      throw new UncheckedIOException(e);
    }
  }

  public static Content content(final Object value) {
    return new Content(new BytesPayload(bytes(value), APPLICATION_JSON));
  }

  public static Response ok(final Object value) {
    return HttpResponses.ok(content(value));
  }

  public static Map<String, Object> read(final Payload payload) throws IOException {
    try (InputStream in = payload.openInputStream()) {
      return read(in);
    }
  }

  public static Map<String, Object> read(final InputStream in) throws IOException {
    Map<String, Object> value = MAPPER.readValue(in, MAP);
    return value != null ? value : new LinkedHashMap<>();
  }

  public static Map<String, Object> read(final byte[] bytes) throws IOException {
    Map<String, Object> value = MAPPER.readValue(bytes, MAP);
    return value != null ? value : new LinkedHashMap<>();
  }

  public static Map<String, Object> revisions(final String reference, final Collection<ConanRevision> revisions) {
    Map<String, Object> json = new LinkedHashMap<>();
    json.put("reference", reference);
    json.put("revisions", revisions.stream().map(ConanRevision::toJson).toList());
    return json;
  }

  public static Map<String, Object> files(final Collection<String> names) {
    Map<String, Object> files = new LinkedHashMap<>();
    names.stream().sorted().forEach(name -> files.put(name, Map.of()));
    return Map.of("files", files);
  }

  @Nullable
  public static String header(final Context context, final String name) {
    for (Map.Entry<String, String> entry : context.getRequest().getHeaders()) {
      if (entry.getKey().equalsIgnoreCase(name)) {
        return entry.getValue();
      }
    }
    return null;
  }

  public static Map<String, Object> results(final List<String> references) {
    return Map.of("results", references);
  }
}
