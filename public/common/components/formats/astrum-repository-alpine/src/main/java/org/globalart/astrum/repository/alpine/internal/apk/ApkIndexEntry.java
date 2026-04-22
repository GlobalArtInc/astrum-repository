package org.globalart.astrum.repository.alpine.internal.apk;

import java.util.ArrayList;
import java.util.List;

public record ApkIndexEntry(String name, String version, String text)
{
  public String key() {
    return name + "-" + version;
  }

  public static List<ApkIndexEntry> parse(final String apkindex) {
    List<ApkIndexEntry> entries = new ArrayList<>();
    StringBuilder text = new StringBuilder();
    String name = null;
    String version = null;
    for (String line : apkindex.split("\n", -1)) {
      if (line.isEmpty()) {
        if (text.length() > 0 && name != null && version != null) {
          entries.add(new ApkIndexEntry(name, version, text.toString()));
        }
        text.setLength(0);
        name = null;
        version = null;
        continue;
      }
      if (line.startsWith("P:")) {
        name = line.substring(2);
      }
      else if (line.startsWith("V:")) {
        version = line.substring(2);
      }
      text.append(line).append('\n');
    }
    if (text.length() > 0 && name != null && version != null) {
      entries.add(new ApkIndexEntry(name, version, text.toString()));
    }
    return entries;
  }

  public static String render(final List<ApkIndexEntry> entries) {
    StringBuilder out = new StringBuilder();
    for (ApkIndexEntry entry : entries) {
      out.append(entry.text()).append('\n');
    }
    return out.toString();
  }
}
