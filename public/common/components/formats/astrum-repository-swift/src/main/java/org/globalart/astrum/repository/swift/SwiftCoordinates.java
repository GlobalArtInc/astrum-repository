package org.globalart.astrum.repository.swift;

import java.util.Locale;
import java.util.Objects;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkArgument;

public final class SwiftCoordinates
{
  private static final Pattern SCOPE = Pattern.compile("[a-zA-Z0-9](?:[a-zA-Z0-9]|-(?=[a-zA-Z0-9])){0,38}");

  private static final Pattern NAME = Pattern.compile("[a-zA-Z0-9](?:[a-zA-Z0-9]|[-_](?=[a-zA-Z0-9])){0,99}");

  public static final String ARCHIVE_EXTENSION = ".zip";

  public static final String JSON_EXTENSION = ".json";

  private final String scope;

  private final String name;

  @Nullable
  private final String version;

  public SwiftCoordinates(final String scope, final String name, @Nullable final String version) {
    checkArgument(isValidScope(scope), "invalid package scope: %s", scope);
    checkArgument(isValidName(name), "invalid package name: %s", name);
    checkArgument(version == null || SwiftVersion.isValid(version), "invalid version: %s", version);
    this.scope = scope;
    this.name = name;
    this.version = version;
  }

  public static boolean isValidScope(@Nullable final String scope) {
    return scope != null && SCOPE.matcher(scope).matches();
  }

  public static boolean isValidName(@Nullable final String name) {
    return name != null && NAME.matcher(name).matches();
  }

  @Nullable
  public static SwiftCoordinates fromIdentifier(@Nullable final String identifier, @Nullable final String version) {
    if (identifier == null) {
      return null;
    }
    int dot = identifier.indexOf('.');
    if (dot < 0) {
      return null;
    }
    String scope = identifier.substring(0, dot);
    String name = identifier.substring(dot + 1);
    if (!isValidScope(scope) || !isValidName(name) || (version != null && !SwiftVersion.isValid(version))) {
      return null;
    }
    return new SwiftCoordinates(scope, name, version);
  }

  public String getScope() {
    return scope;
  }

  public String getName() {
    return name;
  }

  @Nullable
  public String getVersion() {
    return version;
  }

  public String getLowerScope() {
    return scope.toLowerCase(Locale.ROOT);
  }

  public String getLowerName() {
    return name.toLowerCase(Locale.ROOT);
  }

  public String getIdentifier() {
    return scope + "." + name;
  }

  public SwiftCoordinates withVersion(@Nullable final String newVersion) {
    return new SwiftCoordinates(scope, name, newVersion);
  }

  public SwiftCoordinates withPackage(final String newScope, final String newName) {
    return new SwiftCoordinates(newScope, newName, version);
  }

  public boolean samePackage(final String otherScope, final String otherName) {
    return scope.equalsIgnoreCase(otherScope) && name.equalsIgnoreCase(otherName);
  }

  public String getPackagePath() {
    return "/" + scope + "/" + name;
  }

  public String getReleasesPath() {
    return getPackagePath() + JSON_EXTENSION;
  }

  public String getReleasePath() {
    return getPackagePath() + "/" + requireVersion();
  }

  public String getReleaseMetadataPath() {
    return getReleasePath() + JSON_EXTENSION;
  }

  public String getArchivePath() {
    return getReleasePath() + ARCHIVE_EXTENSION;
  }

  public String getManifestPath(final String filename) {
    return getReleasePath() + "/" + filename;
  }

  public String getArchiveFilename() {
    return name + "-" + requireVersion() + ARCHIVE_EXTENSION;
  }

  private String requireVersion() {
    checkArgument(version != null, "no version in %s", this);
    return version;
  }

  @Override
  public boolean equals(final Object other) {
    return other instanceof SwiftCoordinates coordinates
        && scope.equals(coordinates.scope)
        && name.equals(coordinates.name)
        && Objects.equals(version, coordinates.version);
  }

  @Override
  public int hashCode() {
    return Objects.hash(scope, name, version);
  }

  @Override
  public String toString() {
    return version == null ? getIdentifier() : getIdentifier() + " " + version;
  }
}
