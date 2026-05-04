/*
 * Copyright 2019, Imperva, Inc. All rights reserved.
 *
 * This program and the accompanying materials are made available under the
 * terms of the Eclipse Public License Version 1.0, which accompanies this
 * distribution and is available at http://www.eclipse.org/legal/epl-v10.html.
 *
 * Imperva, the Imperva logo, SecureSphere, Incapsula, CounterBreach,
 * ThreatRadar, Camouflage, Attack Analytics, Prevoty and design are trademarks
 * of Imperva, Inc. and its subsidiaries. All other brand or product names are
 * trademarks or registered trademarks of their respective owners.
 */
package org.globalart.astrum.repository.cargo;

import java.util.Locale;
import java.util.Objects;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import static com.google.common.base.Preconditions.checkArgument;
import static com.google.common.base.Preconditions.checkNotNull;

public final class CrateCoordinates
{
  public static final String CRATES_PREFIX = "/crates/";

  public static final String CRATE_EXTENSION = ".crate";

  private static final int MAX_NAME_LENGTH = 64;

  private static final Pattern NAME = Pattern.compile("[A-Za-z_][A-Za-z0-9_-]*");

  private static final Pattern VERSION = Pattern.compile(
      "(0|[1-9]\\d*)\\.(0|[1-9]\\d*)\\.(0|[1-9]\\d*)"
          + "(-[0-9A-Za-z-]+(\\.[0-9A-Za-z-]+)*)?"
          + "(\\+[0-9A-Za-z-]+(\\.[0-9A-Za-z-]+)*)?");

  private final String name;

  private final String version;

  public CrateCoordinates(final String name, final String version) {
    this.name = checkNotNull(name);
    this.version = checkNotNull(version);
    checkArgument(isValidName(name), "Invalid crate name: %s", name);
    checkArgument(isValidVersion(version), "Invalid crate version: %s", version);
  }

  public String getName() {
    return name;
  }

  public String getVersion() {
    return version;
  }

  public String getLowerName() {
    return name.toLowerCase(Locale.ROOT);
  }

  public String getFileBasename() {
    return name + "-" + version;
  }

  public String getFileName() {
    return getFileBasename() + CRATE_EXTENSION;
  }

  public String getAssetPath() {
    return CRATES_PREFIX + name + "/" + version + "/" + getFileName();
  }

  public String getIndexEntryPath() {
    return indexPath(name);
  }

  public static String indexPath(final String crateName) {
    String name = crateName.toLowerCase(Locale.ROOT);
    if (name.length() == 1) {
      return "1/" + name;
    }
    else if (name.length() == 2) {
      return "2/" + name;
    }
    else if (name.length() == 3) {
      return "3/" + name.charAt(0) + "/" + name;
    }
    else {
      return name.substring(0, 2) + "/" + name.substring(2, 4) + "/" + name;
    }
  }

  @Nullable
  public static String nameFromIndexPath(final String path) {
    String relative = path.startsWith("/") ? path.substring(1) : path;
    String[] segments = relative.split("/", -1);
    String name = segments[segments.length - 1];
    if (!isValidName(name) || !indexPath(name).equals(relative.toLowerCase(Locale.ROOT))) {
      return null;
    }
    return name.toLowerCase(Locale.ROOT);
  }

  public static boolean isValidName(@Nullable final String name) {
    return name != null && !name.isEmpty() && name.length() <= MAX_NAME_LENGTH && NAME.matcher(name).matches();
  }

  public static boolean isValidVersion(@Nullable final String version) {
    return version != null && version.length() <= 128 && VERSION.matcher(version).matches();
  }

  @Override
  public boolean equals(final Object o) {
    if (this == o) {
      return true;
    }
    if (!(o instanceof CrateCoordinates that)) {
      return false;
    }
    return name.equals(that.name) && version.equals(that.version);
  }

  @Override
  public int hashCode() {
    return Objects.hash(name, version);
  }

  @Override
  public String toString() {
    return name + "@" + version;
  }
}
