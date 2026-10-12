package org.globalart.astrum.repository.rubygems.internal.index;

import java.io.IOException;
import java.io.OutputStream;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.function.BiConsumer;
import java.util.zip.GZIPOutputStream;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.rubygems.internal.gem.GemPlatform;
import org.globalart.astrum.repository.rubygems.internal.gem.GemVersion;
import org.globalart.astrum.repository.rubygems.internal.marshal.RubyMarshalWriter;

public final class LegacySpecs
{
  public enum Kind
  {
    SPECS("specs"),
    LATEST_SPECS("latest_specs"),
    PRERELEASE_SPECS("prerelease_specs");

    private final String file;

    Kind(final String file) {
      this.file = file;
    }

    public String path() {
      return "/" + file + ".4.8.gz";
    }

    @Nullable
    public static Kind fromPath(final String path) {
      for (Kind kind : values()) {
        if (kind.path().equals(path)) {
          return kind;
        }
      }
      return null;
    }
  }

  public interface Source
  {
    void forEach(BiConsumer<String, List<String>> consumer);
  }

  private LegacySpecs() {
  }

  public static void write(final Kind kind, final Source source, final OutputStream out) throws IOException {
    int[] count = new int[1];
    source.forEach((name, versions) -> count[0] += select(kind, versions).size());
    GZIPOutputStream gzip = new GZIPOutputStream(out, 65536);
    RubyMarshalWriter writer = new RubyMarshalWriter(gzip);
    writer.array(count[0]);
    int[] written = new int[1];
    source.forEach((name, versions) -> {
      for (String versionAndPlatform : select(kind, versions)) {
        int dash = versionAndPlatform.indexOf('-');
        String version = dash < 0 ? versionAndPlatform : versionAndPlatform.substring(0, dash);
        String platform = dash < 0 ? GemPlatform.RUBY : versionAndPlatform.substring(dash + 1);
        writer.array(3).string(name).gemVersion(version).string(platform);
        written[0]++;
      }
    });
    if (written[0] != count[0]) {
      throw new IllegalStateException("Specification index changed while it was written");
    }
    gzip.finish();
    gzip.flush();
  }

  static List<String> select(final Kind kind, final List<String> versionsAndPlatforms) {
    List<String> selected = new ArrayList<>();
    Map<String, String> latest = new LinkedHashMap<>();
    for (String versionAndPlatform : versionsAndPlatforms) {
      if (versionAndPlatform.isEmpty()) {
        continue;
      }
      int dash = versionAndPlatform.indexOf('-');
      String version = dash < 0 ? versionAndPlatform : versionAndPlatform.substring(0, dash);
      if (!GemVersion.isValid(version)) {
        continue;
      }
      boolean prerelease = GemVersion.isPrerelease(version);
      switch (kind) {
        case SPECS -> {
          if (!prerelease) {
            selected.add(versionAndPlatform);
          }
        }
        case PRERELEASE_SPECS -> {
          if (prerelease) {
            selected.add(versionAndPlatform);
          }
        }
        case LATEST_SPECS -> {
          if (!prerelease) {
            String platform = dash < 0 ? GemPlatform.RUBY : versionAndPlatform.substring(dash + 1);
            latest.merge(platform, versionAndPlatform, (current, candidate) ->
                GemVersion.compare(versionOf(candidate), versionOf(current)) > 0 ? candidate : current);
          }
        }
      }
    }
    if (kind == Kind.LATEST_SPECS) {
      selected.addAll(latest.values());
    }
    return selected;
  }

  private static String versionOf(final String versionAndPlatform) {
    int dash = versionAndPlatform.indexOf('-');
    return dash < 0 ? versionAndPlatform : versionAndPlatform.substring(0, dash);
  }
}
