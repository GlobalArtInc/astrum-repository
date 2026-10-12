package org.globalart.astrum.repository.rubygems.internal.gem;

import java.util.regex.Pattern;

import javax.annotation.Nullable;

public record GemCoordinates(String name, String version, String platform)
{
  public static final String GEMS_PREFIX = "/gems/";

  public static final String QUICK_PREFIX = "/quick/Marshal.4.8/";

  public static final String GEM_EXTENSION = ".gem";

  public static final String QUICK_EXTENSION = ".gemspec.rz";

  private static final Pattern NAME = Pattern.compile("[A-Za-z0-9._-]{1,128}");

  private static final Pattern LETTER = Pattern.compile("[A-Za-z]");

  private static final Pattern FILE_NAME = Pattern.compile("[A-Za-z0-9._-]{1,320}");

  public GemCoordinates {
    platform = GemPlatform.normalize(platform);
  }

  public static boolean isValidName(@Nullable final String name) {
    return name != null && NAME.matcher(name).matches() && LETTER.matcher(name).find()
        && !name.startsWith(".") && !name.startsWith("-");
  }

  public static boolean isValidFileName(@Nullable final String fileName) {
    return fileName != null && FILE_NAME.matcher(fileName).matches() && !fileName.startsWith(".");
  }

  public static GemCoordinates of(final GemSpec spec) {
    return new GemCoordinates(spec.name(), spec.version(), spec.platform());
  }

  public boolean isRuby() {
    return GemPlatform.isRuby(platform);
  }

  public String fullName() {
    return isRuby() ? name + "-" + version : name + "-" + version + "-" + platform;
  }

  public String versionAndPlatform() {
    return isRuby() ? version : version + "-" + platform;
  }

  public String fileName() {
    return fullName() + GEM_EXTENSION;
  }

  public String gemPath() {
    return GEMS_PREFIX + fileName();
  }

  public String quickSpecPath() {
    return QUICK_PREFIX + fullName() + QUICK_EXTENSION;
  }

  public static String gemPathForFullName(final String fullName) {
    return GEMS_PREFIX + fullName + GEM_EXTENSION;
  }

  @Override
  public String toString() {
    return fullName();
  }
}
