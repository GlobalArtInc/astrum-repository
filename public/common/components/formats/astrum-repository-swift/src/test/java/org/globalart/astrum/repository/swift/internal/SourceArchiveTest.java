package org.globalart.astrum.repository.swift.internal;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.LinkedHashMap;
import java.util.Map;
import java.util.zip.ZipEntry;
import java.util.zip.ZipOutputStream;

import org.junit.jupiter.api.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;

class SourceArchiveTest
{
  static final String MANIFEST = "// swift-tools-version:5.9\nimport PackageDescription\n";

  static byte[] zip(final Map<String, String> entries) throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (ZipOutputStream zip = new ZipOutputStream(out)) {
      for (Map.Entry<String, String> entry : entries.entrySet()) {
        zip.putNextEntry(new ZipEntry(entry.getKey()));
        if (entry.getValue() != null) {
          zip.write(entry.getValue().getBytes(UTF_8));
        }
        zip.closeEntry();
      }
    }
    return out.toByteArray();
  }

  static byte[] packageZip() throws IOException {
    Map<String, String> entries = new LinkedHashMap<>();
    entries.put("LinkedList/", null);
    entries.put("LinkedList/Package@swift-4.2.swift", "// swift-tools-version:4.2\nimport PackageDescription\n");
    entries.put("LinkedList/Package.swift", MANIFEST);
    entries.put("LinkedList/Sources/LinkedList/LinkedList.swift", "public struct LinkedList {}\n");
    entries.put("LinkedList/Sources/Package.swift", "// not a manifest of the package\n");
    entries.put("__MACOSX/LinkedList/._Package.swift", "junk");
    return zip(entries);
  }

  @Test
  void readsTheManifestsOfTheTopLevelDirectory() throws IOException {
    SourceArchive archive = SourceArchive.read(new ByteArrayInputStream(packageZip()));
    assertThat(archive.getRoot(), is("LinkedList"));
    assertThat(archive.getManifests().keySet(), contains("Package.swift", "Package@swift-4.2.swift"));
    assertThat(archive.getManifest(), is(MANIFEST));
    assertThat(archive.getToolsVersions().get("Package.swift"), is("5.9"));
    assertThat(archive.getToolsVersions().get("Package@swift-4.2.swift"), is("4.2"));
  }

  @Test
  void refusesArchivesWithoutManifest() throws IOException {
    byte[] zip = zip(Map.of("LinkedList/README.md", "hello"));
    SwiftProblem problem = assertThrows(SwiftProblem.class, () -> SourceArchive.read(new ByteArrayInputStream(zip)));
    assertThat(problem.getStatus(), is(422));
    assertThat(problem.getMessage(), containsString("Package.swift"));
  }

  @Test
  void refusesArchivesWithoutSingleTopLevelDirectory() throws IOException {
    byte[] flat = zip(Map.of("Package.swift", MANIFEST));
    assertThat(assertThrows(SwiftProblem.class, () -> SourceArchive.read(new ByteArrayInputStream(flat))).getStatus(),
        is(422));
    Map<String, String> entries = new LinkedHashMap<>();
    entries.put("a/Package.swift", MANIFEST);
    entries.put("b/Package.swift", MANIFEST);
    byte[] twoRoots = zip(entries);
    assertThrows(SwiftProblem.class, () -> SourceArchive.read(new ByteArrayInputStream(twoRoots)));
  }

  @Test
  void refusesSomethingThatIsNotAZip() {
    SwiftProblem problem = assertThrows(SwiftProblem.class,
        () -> SourceArchive.read(new ByteArrayInputStream("not a zip".getBytes(UTF_8))));
    assertThat(problem.getStatus(), is(422));
  }

  @Test
  void refusesEntriesEscapingThePackage() throws IOException {
    byte[] zip = zip(Map.of("LinkedList/../../etc/passwd", "x"));
    assertThrows(SwiftProblem.class, () -> SourceArchive.read(new ByteArrayInputStream(zip)));
  }
}
