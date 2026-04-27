
package org.globalart.astrum.repository.r.internal.util;

import java.nio.ByteBuffer;
import java.nio.charset.CharacterCodingException;
import java.nio.charset.CodingErrorAction;
import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.hash.HashAlgorithm;

import static java.nio.charset.StandardCharsets.ISO_8859_1;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;

public final class RMetadataUtils
{
  public static final List<HashAlgorithm> HASH_ALGORITHMS = List.of(MD5, SHA1, SHA256);

  private static final String CONTINUATION_INDENT = "        ";

  public static Map<String, String> parseDescriptionFile(final byte[] bytes) {
    List<Map<String, String>> records = parseDcf(decode(bytes));
    return records.isEmpty() ? new LinkedHashMap<>() : records.get(0);
  }

  public static String decode(final byte[] bytes) {
    try {
      return UTF_8.newDecoder()
          .onMalformedInput(CodingErrorAction.REPORT)
          .onUnmappableCharacter(CodingErrorAction.REPORT)
          .decode(ByteBuffer.wrap(bytes))
          .toString();
    }
    catch (CharacterCodingException e) {
      return new String(bytes, ISO_8859_1);
    }
  }

  public static List<Map<String, String>> parseDcf(final String text) {
    List<Map<String, String>> records = new ArrayList<>();
    Map<String, String> record = new LinkedHashMap<>();
    String field = null;
    for (String line : text.replace("\r\n", "\n").replace('\r', '\n').split("\n", -1)) {
      if (line.isBlank()) {
        if (!record.isEmpty()) {
          records.add(record);
          record = new LinkedHashMap<>();
        }
        field = null;
      }
      else if (Character.isWhitespace(line.charAt(0))) {
        if (field != null) {
          record.put(field, record.get(field) + "\n" + line.stripTrailing());
        }
      }
      else {
        int colon = line.indexOf(':');
        if (colon > 0) {
          field = line.substring(0, colon).trim();
          record.put(field, line.substring(colon + 1).strip());
        }
        else {
          field = null;
        }
      }
    }
    if (!record.isEmpty()) {
      records.add(record);
    }
    return records;
  }

  public static String writeRecord(final Map<String, String> record, final Collection<String> fields) {
    StringBuilder out = new StringBuilder();
    for (String field : fields) {
      String value = record.get(field);
      if (value == null || value.isBlank()) {
        continue;
      }
      out.append(field).append(':');
      boolean first = true;
      for (String line : value.split("\n")) {
        String trimmed = line.strip();
        if (trimmed.isEmpty()) {
          continue;
        }
        out.append(first ? " " : "\n" + CONTINUATION_INDENT).append(trimmed);
        first = false;
      }
      out.append('\n');
    }
    return out.toString();
  }

  private RMetadataUtils() {
  }
}
