package org.globalart.astrum.repository.rubygems.internal.index;

import java.io.BufferedReader;
import java.io.ByteArrayInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.time.Instant;
import java.time.temporal.ChronoUnit;
import java.util.ArrayList;
import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeSet;
import java.util.function.Consumer;

import com.google.common.hash.Hashing;

public final class CompactIndex
{
  public static final String SEPARATOR = "---";

  private CompactIndex() {
  }

  public static byte[] info(final Collection<InfoEntry> entries) {
    List<InfoEntry> sorted = new ArrayList<>(entries);
    sorted.sort(InfoEntry.ORDER);
    StringBuilder out = new StringBuilder(SEPARATOR).append('\n');
    for (InfoEntry entry : sorted) {
      out.append(entry.toLine()).append('\n');
    }
    return bytes(out);
  }

  public static List<String> infoLines(final byte[] info) {
    List<String> lines = new ArrayList<>();
    forEachBodyLine(new ByteArrayInputStream(info), line -> {
      if (!line.isBlank()) {
        lines.add(line);
      }
    });
    return lines;
  }

  public static List<InfoEntry> parseInfo(final byte[] info) {
    return infoLines(info).stream().map(InfoEntry::parse).toList();
  }

  public static byte[] mergeInfo(final List<byte[]> infos) {
    if (infos.size() == 1) {
      return infos.get(0);
    }
    Map<String, InfoEntry> entries = new LinkedHashMap<>();
    for (byte[] info : infos) {
      for (InfoEntry entry : parseInfo(info)) {
        entries.putIfAbsent(entry.versionAndPlatform(), entry);
      }
    }
    return info(entries.values());
  }

  public static List<String> versionsOf(final byte[] info) {
    return parseInfo(info).stream().sorted(InfoEntry.ORDER).map(InfoEntry::versionAndPlatform).toList();
  }

  public static String md5(final byte[] content) {
    return Hashing.md5().hashBytes(content).toString();
  }

  public static byte[] versions(final Instant createdAt, final Collection<VersionsLine> lines) {
    StringBuilder out = new StringBuilder(header(createdAt));
    for (VersionsLine line : lines) {
      out.append(line.toLine()).append('\n');
    }
    return bytes(out);
  }

  public static String header(final Instant createdAt) {
    return "created_at: " + createdAt.truncatedTo(ChronoUnit.SECONDS) + "\n" + SEPARATOR + "\n";
  }

  public static byte[] names(final Collection<String> names) {
    StringBuilder out = new StringBuilder(SEPARATOR).append('\n');
    for (String name : new TreeSet<>(names)) {
      out.append(name).append('\n');
    }
    return bytes(out);
  }

  public static Set<String> parseNames(final byte[] names) {
    Set<String> result = new LinkedHashSet<>();
    forEachBodyLine(new ByteArrayInputStream(names), line -> {
      String name = line.strip();
      if (!name.isEmpty()) {
        result.add(name);
      }
    });
    return result;
  }

  public static void forEachVersionsLine(final InputStream versions, final Consumer<VersionsLine> consumer) {
    forEachBodyLine(versions, line -> {
      VersionsLine parsed = VersionsLine.parse(line);
      if (parsed != null) {
        consumer.accept(parsed);
      }
    });
  }

  public static void forEachBodyLine(final InputStream in, final Consumer<String> consumer) {
    try (BufferedReader reader = new BufferedReader(new InputStreamReader(in, StandardCharsets.UTF_8), 65536)) {
      List<String> leading = new ArrayList<>();
      boolean body = false;
      String line;
      while ((line = reader.readLine()) != null) {
        if (body) {
          consumer.accept(line);
        }
        else if (SEPARATOR.equals(line.strip())) {
          body = true;
          leading.clear();
        }
        else if (leading.size() < 16) {
          leading.add(line);
        }
        else {
          body = true;
          leading.forEach(consumer);
          leading.clear();
          consumer.accept(line);
        }
      }
      if (!body) {
        leading.forEach(consumer);
      }
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  private static byte[] bytes(final CharSequence text) {
    return text.toString().getBytes(StandardCharsets.UTF_8);
  }
}
