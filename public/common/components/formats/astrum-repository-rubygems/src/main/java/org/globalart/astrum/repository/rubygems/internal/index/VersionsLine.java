package org.globalart.astrum.repository.rubygems.internal.index;

import java.util.List;

import javax.annotation.Nullable;

public record VersionsLine(String name, List<String> versions, String checksum)
{
  public VersionsLine {
    versions = List.copyOf(versions);
  }

  public String toLine() {
    return name + " " + String.join(",", versions) + " " + checksum;
  }

  @Nullable
  public static VersionsLine parse(final String line) {
    String[] parts = line.strip().split(" ", 3);
    if (parts.length < 2 || parts[0].isEmpty()) {
      return null;
    }
    return new VersionsLine(parts[0], List.of(parts[1].split(",")), parts.length == 3 ? parts[2].strip() : "");
  }
}
