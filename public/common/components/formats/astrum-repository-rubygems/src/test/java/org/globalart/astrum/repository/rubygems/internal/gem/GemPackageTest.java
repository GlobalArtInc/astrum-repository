package org.globalart.astrum.repository.rubygems.internal.gem;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.List;
import java.util.Map;

import org.apache.commons.compress.archivers.tar.TarArchiveEntry;
import org.apache.commons.compress.archivers.tar.TarArchiveOutputStream;
import org.junit.jupiter.api.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.junit.jupiter.api.Assertions.assertThrows;

class GemPackageTest
{
  static GemSpec read(final String fileName) throws IOException {
    try (InputStream in = GemPackageTest.class.getResourceAsStream(fileName)) {
      return GemPackage.readSpec(in);
    }
  }

  @Test
  void readsTheSpecificationOfAGem() throws IOException {
    GemSpec spec = read("hello_astrum-1.0.0.gem");

    assertThat(spec.name(), is("hello_astrum"));
    assertThat(spec.version(), is("1.0.0"));
    assertThat(spec.platform(), is("ruby"));
    assertThat(spec.summary(), is("A greeting gem"));
    assertThat(spec.description(), is("Says hello from an Astrum repository."));
    assertThat(spec.authors(), contains("Jane Doe", "John Roe"));
    assertThat(spec.email(), contains("jane@example.com"));
    assertThat(spec.homepage(), is("https://example.com/hello"));
    assertThat(spec.licenses(), contains("MIT"));
    assertThat(spec.metadata(), is(Map.of("source_code_uri", "https://example.com/hello/src")));
    assertThat(spec.dependencies(), contains(
        new GemDependency("rake", "runtime", List.of(">= 10", "< 14")),
        new GemDependency("minitest", "development", List.of("~> 5.0"))));
    assertThat(spec.runtimeDependencies(), contains(new GemDependency("rake", "runtime", List.of(">= 10", "< 14"))));
    assertThat(spec.requiredRubyVersion(), contains(">= 2.7"));
    assertThat(spec.requiredRubygemsVersion(), contains(">= 0"));
    assertThat(spec.rubygemsVersion(), is("3.5.22"));
    assertThat(spec.specificationVersion(), is(4));
    assertThat(spec.date().length(), is(10));
    assertThat(spec.coordinates().fileName(), is("hello_astrum-1.0.0.gem"));
  }

  @Test
  void readsPlatformAndPrereleaseGems() throws IOException {
    GemSpec platform = read("hello_astrum-1.0.0-x86_64-linux.gem");
    assertThat(platform.platform(), is("x86_64-linux"));
    assertThat(platform.coordinates().fileName(), is("hello_astrum-1.0.0-x86_64-linux.gem"));
    assertThat(platform.coordinates().quickSpecPath(),
        is("/quick/Marshal.4.8/hello_astrum-1.0.0-x86_64-linux.gemspec.rz"));

    GemSpec prerelease = read("hello_astrum-1.1.0.pre.gem");
    assertThat(prerelease.version(), is("1.1.0.pre"));
    assertThat(prerelease.coordinates().gemPath(), is("/gems/hello_astrum-1.1.0.pre.gem"));
  }

  @Test
  void roundTripsThroughAttributes() throws IOException {
    GemSpec spec = read("hello_astrum-1.0.0.gem");
    assertThat(GemSpec.fromAttributes(spec.toAttributes()), is(spec));
  }

  @Test
  void rejectsArchivesWithoutMetadata() throws IOException {
    byte[] tar = tar(Map.of("data.tar.gz", new byte[]{1, 2, 3}));
    IllegalArgumentException e = assertThrows(IllegalArgumentException.class,
        () -> GemPackage.readSpec(new ByteArrayInputStream(tar)));
    assertThat(e.getMessage(), is("Not a gem file: metadata.gz is missing"));
  }

  @Test
  void rejectsNonTarContent() {
    assertThrows(Exception.class,
        () -> GemPackage.readSpec(new ByteArrayInputStream("not a gem".getBytes(UTF_8))));
  }

  private static byte[] tar(final Map<String, byte[]> entries) throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (TarArchiveOutputStream tar = new TarArchiveOutputStream(out)) {
      for (Map.Entry<String, byte[]> entry : entries.entrySet()) {
        TarArchiveEntry tarEntry = new TarArchiveEntry(entry.getKey());
        tarEntry.setSize(entry.getValue().length);
        tar.putArchiveEntry(tarEntry);
        tar.write(entry.getValue());
        tar.closeArchiveEntry();
      }
    }
    return out.toByteArray();
  }
}
