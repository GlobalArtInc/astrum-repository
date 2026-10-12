package org.globalart.astrum.repository.swift.internal;

import java.io.IOException;
import java.io.InputStream;
import java.util.Collections;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.Map;
import java.util.Set;
import java.util.zip.ZipEntry;
import java.util.zip.ZipException;
import java.util.zip.ZipInputStream;

import static java.nio.charset.StandardCharsets.UTF_8;

public final class SourceArchive
{
  static final int MAX_MANIFEST_SIZE = 1024 * 1024;

  private static final String MAC_RESOURCES = "__MACOSX";

  private final String root;

  private final Map<String, String> manifests;

  private SourceArchive(final String root, final Map<String, String> manifests) {
    this.root = root;
    this.manifests = Collections.unmodifiableMap(manifests);
  }

  public String getRoot() {
    return root;
  }

  public Map<String, String> getManifests() {
    return manifests;
  }

  public String getManifest() {
    return manifests.get(Manifests.MANIFEST);
  }

  public Map<String, String> getToolsVersions() {
    Map<String, String> versions = new LinkedHashMap<>();
    manifests.forEach((filename, content) -> versions.put(filename,
        Manifests.toolsVersionOrVariant(filename, content)));
    return versions;
  }

  public static SourceArchive read(final InputStream in) throws IOException {
    Set<String> roots = new LinkedHashSet<>();
    Map<String, Map<String, String>> candidates = new LinkedHashMap<>();
    boolean entries = false;
    try (ZipInputStream zip = new ZipInputStream(in, UTF_8)) {
      ZipEntry entry;
      while ((entry = zip.getNextEntry()) != null) {
        entries = true;
        String name = entry.getName().replace('\\', '/');
        while (name.startsWith("/")) {
          name = name.substring(1);
        }
        if (name.isEmpty() || name.equals(MAC_RESOURCES) || name.startsWith(MAC_RESOURCES + "/")) {
          continue;
        }
        if (name.contains("../") || name.equals("..")) {
          throw SwiftProblem.unprocessable("source archive contains an entry outside of the package: " + name);
        }
        int slash = name.indexOf('/');
        if (slash < 0) {
          roots.add(entry.isDirectory() ? name : "");
          continue;
        }
        String first = name.substring(0, slash);
        roots.add(first);
        String rest = name.substring(slash + 1);
        if (!entry.isDirectory() && Manifests.isManifest(rest)) {
          candidates.computeIfAbsent(first, key -> new LinkedHashMap<>()).put(rest, readManifest(zip, name));
        }
      }
    }
    catch (ZipException e) {
      throw SwiftProblem.unprocessable("source archive is not a valid zip archive: " + e.getMessage());
    }
    if (!entries) {
      throw SwiftProblem.unprocessable("source archive is not a valid zip archive");
    }
    if (roots.size() != 1 || roots.contains("")) {
      throw SwiftProblem.unprocessable(
          "source archive must contain the package in a single top-level directory, "
              + "as created by swift package archive-source");
    }
    String root = roots.iterator().next();
    Map<String, String> manifests = candidates.getOrDefault(root, Map.of());
    if (!manifests.containsKey(Manifests.MANIFEST)) {
      throw SwiftProblem.unprocessable("package doesn't contain a valid manifest (Package.swift) file");
    }
    Map<String, String> ordered = new LinkedHashMap<>();
    ordered.put(Manifests.MANIFEST, manifests.get(Manifests.MANIFEST));
    manifests.forEach(ordered::putIfAbsent);
    return new SourceArchive(root, ordered);
  }

  private static String readManifest(final ZipInputStream zip, final String name) throws IOException {
    byte[] bytes = zip.readNBytes(MAX_MANIFEST_SIZE + 1);
    if (bytes.length > MAX_MANIFEST_SIZE) {
      throw SwiftProblem.unprocessable("manifest " + name + " is larger than " + MAX_MANIFEST_SIZE + " bytes");
    }
    return new String(bytes, UTF_8);
  }
}
