package org.globalart.astrum.repository.terraform.internal;

import java.io.BufferedInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.TreeSet;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.terraform.ModuleArchives;
import org.globalart.astrum.repository.terraform.ModuleCoordinates;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformPaths;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.apache.commons.compress.archivers.ArchiveEntry;
import org.apache.commons.compress.archivers.ArchiveInputStream;
import org.apache.commons.compress.archivers.tar.TarArchiveInputStream;
import org.apache.commons.compress.archivers.zip.ZipArchiveInputStream;
import org.apache.commons.compress.compressors.bzip2.BZip2CompressorInputStream;
import org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream;

/**
 * Reads the directory layout of a module package.
 *
 * Terraform installs a registry module from the package location the registry gives and then reads the module
 * from the subdirectory named after {@code //}, literally: it does not expand the glob a registry may write there,
 * such as the {@code *} that selects the single top directory of a GitHub tarball. The repository therefore
 * resolves such a subdirectory against the package it caches, and points packages whose files all sit in one top
 * directory at that directory.
 */
public final class ModulePackages
{
  private static final int MAX_ENTRIES = 100_000;

  private ModulePackages() {
  }

  /**
   * The layout of a package: its directories and whether files lie at its root.
   */
  public record Layout(Set<String> directories, boolean rootFiles)
  {
    /**
     * The one top directory that holds every file, when the root holds no file of its own.
     */
    public Optional<String> singleTopDirectory() {
      if (rootFiles) {
        return Optional.empty();
      }
      List<String> top = directories.stream().filter(directory -> directory.indexOf('/') < 0).toList();
      return top.size() == 1 ? Optional.of(top.get(0)) : Optional.empty();
    }

    /**
     * The directory a subdirectory pattern names, when exactly one directory matches it.
     */
    public Optional<String> resolve(final String pattern) {
      String trimmed = trim(pattern);
      if (!isGlob(trimmed)) {
        return Optional.of(trimmed);
      }
      Pattern regex = Pattern.compile(globToRegex(trimmed));
      List<String> matches = directories.stream().filter(directory -> regex.matcher(directory).matches()).toList();
      return matches.size() == 1 ? Optional.of(matches.get(0)) : Optional.empty();
    }
  }

  /**
   * @return the layout, or empty when the archive type cannot be read
   */
  public static Optional<Layout> layout(final InputStream in, final String extension) throws IOException {
    ArchiveInputStream<? extends ArchiveEntry> archive = open(new BufferedInputStream(in), extension);
    if (archive == null) {
      return Optional.empty();
    }
    Set<String> directories = new TreeSet<>();
    boolean rootFiles = false;
    int count = 0;
    try (archive) {
      ArchiveEntry entry;
      while ((entry = archive.getNextEntry()) != null && count++ < MAX_ENTRIES) {
        String name = trim(entry.getName().replace('\\', '/'));
        if (name.isEmpty() || name.equals("pax_global_header")) {
          continue;
        }
        int slash = name.lastIndexOf('/');
        if (entry.isDirectory()) {
          addWithParents(directories, name);
        }
        else if (slash < 0) {
          rootFiles = true;
        }
        else {
          addWithParents(directories, name.substring(0, slash));
        }
      }
    }
    return Optional.of(new Layout(directories, rootFiles));
  }

  /**
   * The {@code X-Terraform-Get} location of a stored package, relative to the download endpoint so it holds for any
   * URL the repository is reached under.
   */
  public static String location(final ModuleCoordinates module, final FluentAsset archive) {
    String location = TerraformPaths.relative(module.directory() + "/download", archive.path());
    Object subdir = archive.attributes(TerraformContentFacet.ATTRIBUTES_KEY).get(TerraformContentFacet.SUBDIR);
    return subdir instanceof String value && !value.isBlank() ? location + "//" + value : location;
  }

  /**
   * The subdirectory Terraform should read a package from: the pattern a registry gave resolved against the
   * package, else the single top directory of a package without files at its root.
   */
  public static String subdirectory(final TempBlob blob, final String extension, @Nullable final String given) {
    Optional<Layout> layout;
    try (InputStream in = blob.get()) {
      layout = layout(in, extension);
    }
    catch (IOException | RuntimeException e) {
      return given == null ? "" : given;
    }
    if (layout.isEmpty()) {
      return given == null ? "" : given;
    }
    if (given != null && !given.isBlank()) {
      return layout.get().resolve(given).orElse(given);
    }
    return layout.get().singleTopDirectory().orElse("");
  }

  public static boolean isGlob(@Nullable final String subdir) {
    return subdir != null && (subdir.contains("*") || subdir.contains("?") || subdir.contains("["));
  }

  @Nullable
  private static ArchiveInputStream<? extends ArchiveEntry> open(final InputStream in, final String extension)
      throws IOException
  {
    return switch (extension) {
      case ModuleArchives.ZIP -> new ZipArchiveInputStream(in);
      case ModuleArchives.TAR_GZ -> new TarArchiveInputStream(new GzipCompressorInputStream(in));
      case "tar.bz2" -> new TarArchiveInputStream(new BZip2CompressorInputStream(in));
      default -> null;
    };
  }

  private static void addWithParents(final Set<String> directories, final String directory) {
    String current = directory;
    while (!current.isEmpty() && directories.add(current)) {
      int slash = current.lastIndexOf('/');
      current = slash < 0 ? "" : current.substring(0, slash);
    }
  }

  private static String trim(final String path) {
    String value = path;
    while (value.startsWith("./")) {
      value = value.substring(2);
    }
    while (value.startsWith("/")) {
      value = value.substring(1);
    }
    while (value.endsWith("/")) {
      value = value.substring(0, value.length() - 1);
    }
    return value;
  }

  private static String globToRegex(final String glob) {
    StringBuilder regex = new StringBuilder();
    for (char c : glob.toCharArray()) {
      switch (c) {
        case '*' -> regex.append("[^/]*");
        case '?' -> regex.append("[^/]");
        case '[', ']' -> regex.append(c);
        default -> regex.append(Pattern.quote(String.valueOf(c)));
      }
    }
    return regex.toString();
  }
}
