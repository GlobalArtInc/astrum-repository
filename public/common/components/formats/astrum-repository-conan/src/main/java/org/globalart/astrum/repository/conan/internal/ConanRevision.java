package org.globalart.astrum.repository.conan.internal;

import java.time.Instant;
import java.time.OffsetDateTime;
import java.time.ZoneOffset;
import java.time.format.DateTimeFormatter;
import java.time.format.DateTimeParseException;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.Map;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkNotNull;

public record ConanRevision(String revision, OffsetDateTime time)
{
  private static final DateTimeFormatter FORMAT = DateTimeFormatter.ofPattern("yyyy-MM-dd'T'HH:mm:ss.SSSZ");

  public static final Comparator<ConanRevision> NEWEST_FIRST =
      Comparator.comparing(ConanRevision::time).reversed().thenComparing(ConanRevision::revision);

  public ConanRevision {
    checkNotNull(revision);
    checkNotNull(time);
  }

  public String isoTime() {
    return FORMAT.format(time.withOffsetSameInstant(ZoneOffset.UTC));
  }

  public Map<String, Object> toJson() {
    Map<String, Object> json = new LinkedHashMap<>();
    json.put("revision", revision);
    json.put("time", isoTime());
    return json;
  }

  @Nullable
  public static ConanRevision fromJson(@Nullable final Object value) {
    if (!(value instanceof Map<?, ?> map) || !(map.get("revision") instanceof String revision)) {
      return null;
    }
    OffsetDateTime time = parseTime(map.get("time") instanceof String text ? text : null);
    return new ConanRevision(revision, time);
  }

  public static OffsetDateTime parseTime(@Nullable final String text) {
    if (text == null || text.isBlank()) {
      return OffsetDateTime.ofInstant(Instant.EPOCH, ZoneOffset.UTC);
    }
    try {
      return OffsetDateTime.parse(text, FORMAT);
    }
    catch (DateTimeParseException e) {
      try {
        return OffsetDateTime.parse(text);
      }
      catch (DateTimeParseException e2) {
        return OffsetDateTime.ofInstant(Instant.EPOCH, ZoneOffset.UTC);
      }
    }
  }
}
