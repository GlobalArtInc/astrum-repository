package org.globalart.astrum.repository.rubygems.internal.index;

import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

import org.globalart.astrum.repository.rubygems.internal.gem.GemDependency;
import org.globalart.astrum.repository.rubygems.internal.gem.GemPlatform;
import org.globalart.astrum.repository.rubygems.internal.gem.GemRequirements;
import org.globalart.astrum.repository.rubygems.internal.gem.GemSpec;
import org.globalart.astrum.repository.rubygems.internal.gem.GemVersion;

public record InfoEntry(
    String version,
    String platform,
    List<GemDependency> dependencies,
    String checksum,
    List<String> requiredRubyVersion,
    List<String> requiredRubygemsVersion)
{
  public static final Comparator<InfoEntry> ORDER = Comparator
      .comparing(InfoEntry::version, GemVersion.COMPARATOR)
      .thenComparing(entry -> GemPlatform.isRuby(entry.platform()) ? "" : entry.platform());

  public InfoEntry {
    platform = GemPlatform.normalize(platform);
    dependencies = List.copyOf(dependencies);
    requiredRubyVersion = List.copyOf(requiredRubyVersion);
    requiredRubygemsVersion = List.copyOf(requiredRubygemsVersion);
  }

  public static InfoEntry of(final GemSpec spec, final String checksum) {
    return new InfoEntry(spec.version(), spec.platform(), spec.runtimeDependencies(), checksum,
        spec.requiredRubyVersion(), spec.requiredRubygemsVersion());
  }

  public String versionAndPlatform() {
    return GemPlatform.isRuby(platform) ? version : version + "-" + platform;
  }

  public String toLine() {
    StringBuilder line = new StringBuilder(versionAndPlatform()).append(' ');
    List<GemDependency> sorted = new ArrayList<>(dependencies);
    sorted.sort(Comparator.comparing(GemDependency::name));
    for (int i = 0; i < sorted.size(); i++) {
      if (i > 0) {
        line.append(',');
      }
      GemDependency dependency = sorted.get(i);
      line.append(dependency.name()).append(':').append(String.join("&", sorted(dependency.requirements())));
    }
    line.append('|');
    List<String> requirements = new ArrayList<>();
    if (checksum != null && !checksum.isEmpty()) {
      requirements.add("checksum:" + checksum);
    }
    if (!GemRequirements.isAny(requiredRubyVersion)) {
      requirements.add("ruby:" + String.join("&", sorted(requiredRubyVersion)));
    }
    if (!GemRequirements.isAny(requiredRubygemsVersion)) {
      requirements.add("rubygems:" + String.join("&", sorted(requiredRubygemsVersion)));
    }
    line.append(String.join(",", requirements));
    return line.toString();
  }

  private static List<String> sorted(final List<String> requirements) {
    return requirements.stream().sorted().toList();
  }

  public static InfoEntry parse(final String line) {
    String trimmed = line.strip();
    int space = trimmed.indexOf(' ');
    String versionAndPlatform = space < 0 ? trimmed : trimmed.substring(0, space);
    String rest = space < 0 ? "" : trimmed.substring(space + 1);
    int dash = versionAndPlatform.indexOf('-');
    String version = dash < 0 ? versionAndPlatform : versionAndPlatform.substring(0, dash);
    String platform = dash < 0 ? GemPlatform.RUBY : versionAndPlatform.substring(dash + 1);
    int bar = rest.indexOf('|');
    String dependencyPart = bar < 0 ? rest : rest.substring(0, bar);
    String requirementPart = bar < 0 ? "" : rest.substring(bar + 1);
    List<GemDependency> dependencies = new ArrayList<>();
    for (String dependency : dependencyPart.split(",")) {
      if (dependency.isBlank()) {
        continue;
      }
      int colon = dependency.indexOf(':');
      String name = colon < 0 ? dependency.strip() : dependency.substring(0, colon).strip();
      List<String> requirements = colon < 0 ? List.of() : List.of(dependency.substring(colon + 1).split("&"));
      dependencies.add(new GemDependency(name, GemDependency.RUNTIME, requirements));
    }
    String checksum = null;
    List<String> ruby = List.of();
    List<String> rubygems = List.of();
    for (String requirement : requirementPart.split(",")) {
      int colon = requirement.indexOf(':');
      if (colon < 0) {
        continue;
      }
      String key = requirement.substring(0, colon).strip();
      String value = requirement.substring(colon + 1);
      switch (key) {
        case "checksum" -> checksum = value.strip();
        case "ruby" -> ruby = List.of(value.split("&"));
        case "rubygems" -> rubygems = List.of(value.split("&"));
        default -> {
        }
      }
    }
    return new InfoEntry(version, platform, dependencies, checksum, ruby, rubygems);
  }
}
