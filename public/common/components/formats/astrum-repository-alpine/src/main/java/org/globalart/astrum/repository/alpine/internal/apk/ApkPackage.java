package org.globalart.astrum.repository.alpine.internal.apk;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

public record ApkPackage(
    String checksum,
    String name,
    String version,
    @Nullable String arch,
    long size,
    long installedSize,
    String description,
    String url,
    String license,
    @Nullable String origin,
    @Nullable String maintainer,
    @Nullable String buildTime,
    @Nullable String commit,
    @Nullable String providerPriority,
    @Nullable String depends,
    @Nullable String provides,
    @Nullable String installIf)
{
  public static final String CHECKSUM = "checksum";

  public static final String NAME = "name";

  public static final String VERSION = "version";

  public static final String ARCH = "arch";

  public static final String SIZE = "size";

  public static final String INSTALLED_SIZE = "installedSize";

  public static final String DESCRIPTION = "description";

  public static final String URL = "url";

  public static final String LICENSE = "license";

  public static final String ORIGIN = "origin";

  public static final String MAINTAINER = "maintainer";

  public static final String BUILD_TIME = "buildTime";

  public static final String COMMIT = "commit";

  public static final String PROVIDER_PRIORITY = "providerPriority";

  public static final String DEPENDS = "depends";

  public static final String PROVIDES = "provides";

  public static final String INSTALL_IF = "installIf";

  private static final Pattern NAME_PATTERN = Pattern.compile("[A-Za-z0-9][A-Za-z0-9+._-]*");

  private static final Pattern VERSION_PATTERN = Pattern.compile("[0-9][A-Za-z0-9._+~]*(-r[0-9]+)?");

  public static ApkPackage fromPkgInfo(
      final Map<String, List<String>> info,
      final String checksum,
      final long fileSize)
  {
    String name = single(info, "pkgname");
    String version = single(info, "pkgver");
    if (name == null || !NAME_PATTERN.matcher(name).matches()) {
      throw new IllegalArgumentException("Invalid or missing pkgname in .PKGINFO: " + name);
    }
    if (version == null || !VERSION_PATTERN.matcher(version).matches()) {
      throw new IllegalArgumentException("Invalid or missing pkgver in .PKGINFO: " + version);
    }
    return new ApkPackage(
        checksum,
        name,
        version,
        single(info, "arch"),
        fileSize,
        parseLong(single(info, "size")),
        orEmpty(single(info, "pkgdesc")),
        orEmpty(single(info, "url")),
        orEmpty(single(info, "license")),
        single(info, "origin"),
        single(info, "maintainer"),
        nonZero(single(info, "builddate")),
        single(info, "commit"),
        nonZero(single(info, "provider_priority")),
        joined(info, "depend"),
        joined(info, "provides"),
        joined(info, "install_if"));
  }

  public static ApkPackage fromAttributes(final Map<String, Object> attributes, final long fileSize) {
    return new ApkPackage(
        string(attributes, CHECKSUM),
        string(attributes, NAME),
        string(attributes, VERSION),
        string(attributes, ARCH),
        fileSize,
        attributes.get(INSTALLED_SIZE) instanceof Number number ? number.longValue() :
            parseLong(string(attributes, INSTALLED_SIZE)),
        orEmpty(string(attributes, DESCRIPTION)),
        orEmpty(string(attributes, URL)),
        orEmpty(string(attributes, LICENSE)),
        string(attributes, ORIGIN),
        string(attributes, MAINTAINER),
        string(attributes, BUILD_TIME),
        string(attributes, COMMIT),
        string(attributes, PROVIDER_PRIORITY),
        string(attributes, DEPENDS),
        string(attributes, PROVIDES),
        string(attributes, INSTALL_IF));
  }

  public Map<String, Object> toAttributes() {
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(CHECKSUM, checksum);
    attributes.put(NAME, name);
    attributes.put(VERSION, version);
    putIfPresent(attributes, ARCH, arch);
    attributes.put(SIZE, size);
    attributes.put(INSTALLED_SIZE, installedSize);
    attributes.put(DESCRIPTION, description);
    attributes.put(URL, url);
    attributes.put(LICENSE, license);
    putIfPresent(attributes, ORIGIN, origin);
    putIfPresent(attributes, MAINTAINER, maintainer);
    putIfPresent(attributes, BUILD_TIME, buildTime);
    putIfPresent(attributes, COMMIT, commit);
    putIfPresent(attributes, PROVIDER_PRIORITY, providerPriority);
    putIfPresent(attributes, DEPENDS, depends);
    putIfPresent(attributes, PROVIDES, provides);
    putIfPresent(attributes, INSTALL_IF, installIf);
    return attributes;
  }

  public String fileName() {
    return name + "-" + version + ".apk";
  }

  public boolean isNoarch() {
    return "noarch".equals(arch);
  }

  public ApkIndexEntry indexEntry() {
    StringBuilder text = new StringBuilder();
    line(text, 'C', checksum);
    line(text, 'P', name);
    line(text, 'V', version);
    optional(text, 'A', arch);
    line(text, 'S', Long.toString(size));
    line(text, 'I', Long.toString(installedSize));
    line(text, 'T', description);
    line(text, 'U', url);
    line(text, 'L', license);
    optional(text, 'o', origin);
    optional(text, 'm', maintainer);
    optional(text, 't', buildTime);
    optional(text, 'c', commit);
    optional(text, 'k', providerPriority);
    optional(text, 'D', depends);
    optional(text, 'p', provides);
    optional(text, 'i', installIf);
    return new ApkIndexEntry(name, version, text.toString());
  }

  private static void line(final StringBuilder text, final char field, final String value) {
    text.append(field).append(':').append(value.replace('\n', ' ')).append('\n');
  }

  private static void optional(final StringBuilder text, final char field, @Nullable final String value) {
    if (value != null && !value.isEmpty()) {
      line(text, field, value);
    }
  }

  private static void putIfPresent(final Map<String, Object> attributes, final String key, @Nullable final String value) {
    if (value != null && !value.isEmpty()) {
      attributes.put(key, value);
    }
  }

  @Nullable
  private static String single(final Map<String, List<String>> info, final String key) {
    List<String> values = info.get(key);
    return values == null || values.isEmpty() ? null : values.get(0);
  }

  @Nullable
  private static String joined(final Map<String, List<String>> info, final String key) {
    List<String> values = info.get(key);
    if (values == null) {
      return null;
    }
    String result = String.join(" ", values.stream().filter(value -> !value.isEmpty()).toList()).strip();
    return result.isEmpty() ? null : result;
  }

  @Nullable
  private static String nonZero(@Nullable final String value) {
    return value == null || value.isEmpty() || "0".equals(value) ? null : value;
  }

  private static String orEmpty(@Nullable final String value) {
    return value == null ? "" : value;
  }

  private static long parseLong(@Nullable final String value) {
    if (value == null || value.isEmpty()) {
      return 0L;
    }
    try {
      return Long.parseLong(value);
    }
    catch (NumberFormatException e) {
      return 0L;
    }
  }

  @Nullable
  private static String string(final Map<String, Object> attributes, final String key) {
    Object value = attributes.get(key);
    return value == null ? null : value.toString();
  }
}
