package org.globalart.astrum.repository.swift.internal;

import java.io.IOException;
import java.time.Instant;
import java.time.OffsetDateTime;
import java.time.format.DateTimeFormatter;
import java.time.format.DateTimeParseException;
import java.time.temporal.ChronoUnit;
import java.util.ArrayList;
import java.util.Iterator;
import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.swift.SwiftCoordinates;

import com.fasterxml.jackson.core.type.TypeReference;
import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

import static org.globalart.astrum.repository.swift.internal.SwiftResponses.APPLICATION_ZIP;
import static org.globalart.astrum.repository.swift.internal.SwiftResponses.MAPPER;

public final class ReleaseMetadata
{
  public static final String SOURCE_ARCHIVE = "source-archive";

  public static final String ORIGINAL_PUBLICATION_TIME = "originalPublicationTime";

  private static final List<String> TEXT_FIELDS = List.of("description", "licenseURL", "readmeURL",
      ORIGINAL_PUBLICATION_TIME);

  private static final List<String> PERSON_FIELDS = List.of("name", "email", "description", "url");

  private ReleaseMetadata() {
  }

  public record Signing(String signatureBase64Encoded, String signatureFormat)
  {
  }

  public static ObjectNode parse(@Nullable final byte[] bytes) {
    if (bytes == null || bytes.length == 0) {
      return MAPPER.createObjectNode();
    }
    JsonNode node;
    try {
      node = MAPPER.readTree(bytes);
    }
    catch (IOException e) {
      throw SwiftProblem.unprocessable("invalid JSON provided for release metadata");
    }
    if (node == null || node.isMissingNode()) {
      return MAPPER.createObjectNode();
    }
    if (!(node instanceof ObjectNode object)) {
      throw SwiftProblem.unprocessable("invalid JSON provided for release metadata: expected an object");
    }
    validate(object);
    return object;
  }

  static void validate(final ObjectNode metadata) {
    for (String field : TEXT_FIELDS) {
      requireText(metadata, field, field);
    }
    JsonNode time = metadata.get(ORIGINAL_PUBLICATION_TIME);
    if (time != null && !time.isNull() && parseTime(time.asText()) == null) {
      throw invalid(ORIGINAL_PUBLICATION_TIME + " must be an ISO 8601 date-time");
    }
    JsonNode urls = metadata.get("repositoryURLs");
    if (urls != null && !urls.isNull()) {
      if (!urls.isArray()) {
        throw invalid("repositoryURLs must be an array of strings");
      }
      for (JsonNode url : urls) {
        if (!url.isTextual()) {
          throw invalid("repositoryURLs must be an array of strings");
        }
      }
    }
    JsonNode author = metadata.get("author");
    if (author != null && !author.isNull()) {
      validatePerson(author, "author");
      JsonNode organization = author.get("organization");
      if (organization != null && !organization.isNull()) {
        validatePerson(organization, "author.organization");
      }
    }
  }

  private static void validatePerson(final JsonNode person, final String path) {
    if (!person.isObject()) {
      throw invalid(path + " must be an object");
    }
    JsonNode name = person.get("name");
    if (name == null || !name.isTextual() || name.asText().isBlank()) {
      throw invalid(path + ".name is required");
    }
    for (String field : PERSON_FIELDS) {
      requireText(person, field, path + "." + field);
    }
  }

  private static void requireText(final JsonNode node, final String field, final String path) {
    JsonNode value = node.get(field);
    if (value != null && !value.isNull() && !value.isTextual()) {
      throw invalid(path + " must be a string");
    }
  }

  private static SwiftProblem invalid(final String detail) {
    return SwiftProblem.unprocessable("invalid JSON provided for release metadata: " + detail);
  }

  public static List<String> repositoryUrls(@Nullable final JsonNode metadata) {
    List<String> urls = new ArrayList<>();
    JsonNode node = metadata == null ? null : metadata.get("repositoryURLs");
    if (node != null && node.isArray()) {
      node.forEach(url -> {
        if (url.isTextual() && !url.asText().isBlank()) {
          urls.add(url.asText().strip());
        }
      });
    }
    return urls;
  }

  @Nullable
  public static String text(@Nullable final JsonNode node, final String field) {
    JsonNode value = node == null ? null : node.get(field);
    return value != null && value.isTextual() ? value.asText() : null;
  }

  public static Map<String, Object> toMap(final JsonNode node) {
    return MAPPER.convertValue(node, new TypeReference<Map<String, Object>>() { });
  }

  public static ObjectNode toNode(@Nullable final Object value) {
    if (value == null) {
      return MAPPER.createObjectNode();
    }
    JsonNode node = MAPPER.valueToTree(value);
    return node instanceof ObjectNode object ? object : MAPPER.createObjectNode();
  }

  public static ObjectNode release(
      final SwiftCoordinates coordinates,
      @Nullable final String checksum,
      @Nullable final Signing signing,
      @Nullable final ObjectNode metadata,
      @Nullable final String publishedAt)
  {
    ObjectNode release = MAPPER.createObjectNode();
    release.put("id", coordinates.getIdentifier());
    release.put("version", coordinates.getVersion());
    ArrayNode resources = release.putArray("resources");
    ObjectNode archive = resources.addObject();
    archive.put("name", SOURCE_ARCHIVE);
    archive.put("type", APPLICATION_ZIP);
    if (checksum != null) {
      archive.put("checksum", checksum);
    }
    if (signing != null) {
      ObjectNode node = archive.putObject("signing");
      node.put("signatureBase64Encoded", signing.signatureBase64Encoded());
      node.put("signatureFormat", signing.signatureFormat());
    }
    release.set("metadata", normalizeTimes(metadata == null ? MAPPER.createObjectNode() : metadata.deepCopy()));
    if (publishedAt != null) {
      release.put("publishedAt", publishedAt);
    }
    return release;
  }

  static ObjectNode normalizeTimes(final ObjectNode metadata) {
    String time = text(metadata, ORIGINAL_PUBLICATION_TIME);
    if (time != null) {
      String normalized = parseTime(time);
      if (normalized != null) {
        metadata.put(ORIGINAL_PUBLICATION_TIME, normalized);
      }
    }
    return metadata;
  }

  @Nullable
  public static Signing signing(@Nullable final JsonNode release) {
    if (release == null) {
      return null;
    }
    JsonNode resources = release.get("resources");
    if (resources == null || !resources.isArray()) {
      return null;
    }
    for (Iterator<JsonNode> it = resources.elements(); it.hasNext(); ) {
      JsonNode resource = it.next();
      if (SOURCE_ARCHIVE.equals(text(resource, "name"))) {
        JsonNode signing = resource.get("signing");
        String signature = text(signing, "signatureBase64Encoded");
        String format = text(signing, "signatureFormat");
        return signature != null && format != null ? new Signing(signature, format) : null;
      }
    }
    return null;
  }

  @Nullable
  public static String checksum(@Nullable final JsonNode release) {
    JsonNode resources = release == null ? null : release.get("resources");
    if (resources != null && resources.isArray()) {
      for (JsonNode resource : resources) {
        if (SOURCE_ARCHIVE.equals(text(resource, "name"))) {
          return text(resource, "checksum");
        }
      }
    }
    return null;
  }

  public static String now() {
    return format(Instant.now());
  }

  public static String format(final Instant instant) {
    return DateTimeFormatter.ISO_INSTANT.format(instant.truncatedTo(ChronoUnit.SECONDS));
  }

  @Nullable
  static String parseTime(final String value) {
    try {
      return format(OffsetDateTime.parse(value, DateTimeFormatter.ISO_OFFSET_DATE_TIME).toInstant());
    }
    catch (DateTimeParseException e) {
      try {
        return format(Instant.parse(value));
      }
      catch (DateTimeParseException ignored) {
        return null;
      }
    }
  }
}
