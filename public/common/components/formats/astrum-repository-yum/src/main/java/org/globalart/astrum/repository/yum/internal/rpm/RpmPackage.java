package org.globalart.astrum.repository.yum.internal.rpm;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

public record RpmPackage(
    String name,
    String epoch,
    String version,
    String release,
    String arch,
    @Nullable String summary,
    @Nullable String description,
    @Nullable String packager,
    @Nullable String url,
    long buildTime,
    long installedSize,
    long archiveSize,
    @Nullable String license,
    @Nullable String vendor,
    @Nullable String group,
    @Nullable String buildHost,
    @Nullable String sourceRpm,
    long headerStart,
    long headerEnd,
    List<RpmDependency> provides,
    List<RpmDependency> requires,
    List<RpmDependency> conflicts,
    List<RpmDependency> obsoletes,
    List<RpmDependency> recommends,
    List<RpmDependency> suggests,
    List<RpmDependency> supplements,
    List<RpmDependency> enhances,
    List<RpmFile> files,
    List<RpmChangelog> changelogs)
{
  public static final String NAME = "name";

  public static final String EPOCH = "epoch";

  public static final String VERSION = "version";

  public static final String RELEASE = "release";

  public static final String ARCHITECTURE = "architecture";

  public static final String SUMMARY = "summary";

  public static final String DESCRIPTION = "description";

  public static final String LICENSE = "license";

  public static final String URL = "url";

  public static final String VENDOR = "vendor";

  public static final String GROUP = "group";

  public static final String SOURCE_RPM = "sourceRpm";

  public static final String BUILD_TIME = "buildTime";

  public boolean isSource() {
    return "src".equals(arch) || "nosrc".equals(arch);
  }

  public String versionRelease() {
    return version + "-" + release;
  }

  public String componentVersion() {
    return "0".equals(epoch) ? versionRelease() : epoch + ":" + versionRelease();
  }

  public String fileName() {
    return name + "-" + versionRelease() + "." + arch + ".rpm";
  }

  public Map<String, Object> toAttributes() {
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(NAME, name);
    attributes.put(EPOCH, epoch);
    attributes.put(VERSION, version);
    attributes.put(RELEASE, release);
    attributes.put(ARCHITECTURE, arch);
    putIfPresent(attributes, SUMMARY, summary);
    putIfPresent(attributes, DESCRIPTION, description);
    putIfPresent(attributes, LICENSE, license);
    putIfPresent(attributes, URL, url);
    putIfPresent(attributes, VENDOR, vendor);
    putIfPresent(attributes, GROUP, group);
    putIfPresent(attributes, SOURCE_RPM, sourceRpm);
    if (buildTime > 0) {
      attributes.put(BUILD_TIME, buildTime);
    }
    return attributes;
  }

  private static void putIfPresent(final Map<String, Object> attributes, final String key, @Nullable final String value) {
    if (value != null && !value.isEmpty()) {
      attributes.put(key, value);
    }
  }
}
