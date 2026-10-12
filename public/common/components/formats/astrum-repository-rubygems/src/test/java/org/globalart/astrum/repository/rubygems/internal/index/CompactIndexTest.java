package org.globalart.astrum.repository.rubygems.internal.index;

import java.io.ByteArrayInputStream;
import java.time.Instant;
import java.util.ArrayList;
import java.util.List;

import org.globalart.astrum.repository.rubygems.internal.gem.GemDependency;

import org.junit.jupiter.api.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;

class CompactIndexTest
{
  private static final InfoEntry RELEASE = new InfoEntry("1.0.0", "ruby",
      List.of(new GemDependency("rake", "runtime", List.of(">= 10", "< 14")),
          new GemDependency("json", "runtime", List.of(">= 0"))),
      "abc", List.of(">= 2.7"), List.of(">= 0"));

  private static final InfoEntry PLATFORM = new InfoEntry("1.0.0", "x86_64-linux", List.of(), "def",
      List.of(">= 0"), List.of("> 1.3.1"));

  private static final InfoEntry PRERELEASE = new InfoEntry("1.1.0.pre", "ruby", List.of(), "123",
      List.of(">= 0"), List.of(">= 0"));

  @Test
  void writesInfoLines() {
    assertThat(RELEASE.toLine(), is("1.0.0 json:>= 0,rake:< 14&>= 10|checksum:abc,ruby:>= 2.7"));
    assertThat(PLATFORM.toLine(), is("1.0.0-x86_64-linux |checksum:def,rubygems:> 1.3.1"));
  }

  @Test
  void writesInfoFilesOrderedByVersion() {
    byte[] info = CompactIndex.info(List.of(PRERELEASE, PLATFORM, RELEASE));

    assertThat(new String(info, UTF_8), is("---\n"
        + "1.0.0 json:>= 0,rake:< 14&>= 10|checksum:abc,ruby:>= 2.7\n"
        + "1.0.0-x86_64-linux |checksum:def,rubygems:> 1.3.1\n"
        + "1.1.0.pre |checksum:123\n"));
  }

  @Test
  void parsesInfoLines() {
    assertThat(InfoEntry.parse(RELEASE.toLine()), is(new InfoEntry("1.0.0", "ruby",
        List.of(new GemDependency("json", "runtime", List.of(">= 0")),
            new GemDependency("rake", "runtime", List.of("< 14", ">= 10"))),
        "abc", List.of(">= 2.7"), List.of())));
    assertThat(InfoEntry.parse("2.0.0 |checksum:x").versionAndPlatform(), is("2.0.0"));
    assertThat(InfoEntry.parse("2.0.0-java").platform(), is("java"));
  }

  @Test
  void mergesInfoFilesKeepingTheFirstMember() {
    byte[] first = CompactIndex.info(List.of(RELEASE));
    byte[] second = CompactIndex.info(List.of(
        new InfoEntry("1.0.0", "ruby", List.of(), "other", List.of(), List.of()), PRERELEASE));

    byte[] merged = CompactIndex.mergeInfo(List.of(first, second));

    assertThat(CompactIndex.infoLines(merged), contains(RELEASE.toLine(), PRERELEASE.toLine()));
    assertThat(CompactIndex.mergeInfo(List.of(first)), is(first));
    assertThat(CompactIndex.versionsOf(merged), contains("1.0.0", "1.1.0.pre"));
  }

  @Test
  void writesAndReadsVersionsFiles() {
    byte[] versions = CompactIndex.versions(Instant.parse("2026-10-12T10:15:30.25Z"), List.of(
        new VersionsLine("a", List.of("1.0.0", "1.1.0-java"), "0cc175b9c0f1b6a831c399e269772661"),
        new VersionsLine("b", List.of("2.0"), "92eb5ffee6ae2fec3ad71c777531578f")));

    assertThat(new String(versions, UTF_8), is("created_at: 2026-10-12T10:15:30Z\n---\n"
        + "a 1.0.0,1.1.0-java 0cc175b9c0f1b6a831c399e269772661\n"
        + "b 2.0 92eb5ffee6ae2fec3ad71c777531578f\n"));
    List<VersionsLine> lines = new ArrayList<>();
    CompactIndex.forEachVersionsLine(new ByteArrayInputStream(versions), lines::add);
    assertThat(lines.get(0), is(new VersionsLine("a", List.of("1.0.0", "1.1.0-java"),
        "0cc175b9c0f1b6a831c399e269772661")));
    assertThat(lines.size(), is(2));
  }

  @Test
  void readsBodiesWithoutSeparator() {
    List<String> lines = new ArrayList<>();
    CompactIndex.forEachBodyLine(new ByteArrayInputStream("a 1.0 x\nb 2.0 y\n".getBytes(UTF_8)), lines::add);
    assertThat(lines, contains("a 1.0 x", "b 2.0 y"));
  }

  @Test
  void writesAndReadsNames() {
    byte[] names = CompactIndex.names(List.of("rake", "astrum", "rack"));

    assertThat(new String(names, UTF_8), is("---\nastrum\nrack\nrake\n"));
    assertThat(CompactIndex.parseNames(names), contains("astrum", "rack", "rake"));
  }

  @Test
  void computesMd5() {
    assertThat(CompactIndex.md5("a".getBytes(UTF_8)), is("0cc175b9c0f1b6a831c399e269772661"));
  }
}
