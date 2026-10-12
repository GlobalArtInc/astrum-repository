package org.globalart.astrum.repository.swift.internal;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

public final class Manifests
{
  public static final String MANIFEST = "Package.swift";

  private static final Pattern VARIANT = Pattern.compile("Package@swift-(\\d{1,9}(?:\\.\\d{1,9}){0,2})\\.swift");

  private static final Pattern TOOLS_VERSION = Pattern.compile(
      "^\\s*//\\s*swift-tools-version\\s*:\\s*(\\d+(?:\\.\\d+){0,2})", Pattern.CASE_INSENSITIVE);

  private static final Pattern SWIFT_VERSION = Pattern.compile("\\d{1,9}(?:\\.\\d{1,9}){0,2}");

  private Manifests() {
  }

  public static boolean isManifest(@Nullable final String filename) {
    return MANIFEST.equals(filename) || isVariant(filename);
  }

  public static boolean isVariant(@Nullable final String filename) {
    return filename != null && VARIANT.matcher(filename).matches();
  }

  public static Optional<String> variantVersion(@Nullable final String filename) {
    if (filename == null) {
      return Optional.empty();
    }
    Matcher matcher = VARIANT.matcher(filename);
    return matcher.matches() ? Optional.of(matcher.group(1)) : Optional.empty();
  }

  public static Optional<String> toolsVersion(@Nullable final String manifest) {
    if (manifest == null) {
      return Optional.empty();
    }
    String content = manifest.startsWith("﻿") ? manifest.substring(1) : manifest;
    int newline = content.indexOf('\n');
    String firstLine = newline < 0 ? content : content.substring(0, newline);
    Matcher matcher = TOOLS_VERSION.matcher(firstLine);
    return matcher.find() ? Optional.of(matcher.group(1)) : Optional.empty();
  }

  public static String toolsVersionOrVariant(final String filename, @Nullable final String manifest) {
    return toolsVersion(manifest).or(() -> variantVersion(filename)).orElse("");
  }

  public static boolean isSwiftVersion(@Nullable final String value) {
    return value != null && SWIFT_VERSION.matcher(value).matches();
  }

  public static Optional<String> resolve(final Iterable<String> variants, @Nullable final String swiftVersion) {
    if (!isSwiftVersion(swiftVersion)) {
      return Optional.empty();
    }
    List<Integer> requested = parts(swiftVersion);
    for (String filename : variants) {
      Optional<String> version = variantVersion(filename);
      if (version.isPresent() && parts(version.get()).equals(requested)) {
        return Optional.of(filename);
      }
    }
    return Optional.empty();
  }

  public static Optional<String> resolve(final Map<String, String> variants, @Nullable final String swiftVersion) {
    return resolve(variants.keySet(), swiftVersion);
  }

  private static List<Integer> parts(final String version) {
    List<Integer> parts = new ArrayList<>();
    for (String part : version.split("\\.")) {
      parts.add(Integer.parseInt(part));
    }
    while (parts.size() < 3) {
      parts.add(0);
    }
    return parts;
  }
}
