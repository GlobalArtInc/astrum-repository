package org.globalart.astrum.repository.rubygems.internal.gem;

import java.io.ByteArrayInputStream;
import java.util.List;

import org.junit.jupiter.api.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.junit.jupiter.api.Assertions.assertThrows;

class GemSpecParserTest
{
  private static GemSpec parse(final String yaml) {
    return GemSpecParser.parse(new ByteArrayInputStream(yaml.getBytes(UTF_8)));
  }

  @Test
  void readsOldStyleSpecifications() {
    GemSpec spec = parse("""
        --- !ruby/object:Gem::Specification
        name: legacy
        version: !ruby/object:Gem::Version
          version: 0.9.1
        platform: !ruby/object:Gem::Platform
          cpu: x86
          os: mswin32
          version: '60'
        authors: Old Author
        email: !binary |-
          b2xkQGV4YW1wbGUuY29t
        license: GPL-2.0
        date: 2009-03-01 00:00:00 +01:00
        dependencies:
        - !ruby/object:Gem::Dependency
          name: json
          version_requirements: !ruby/object:Gem::Requirement
            requirements:
            - - ">"
              - !ruby/object:Gem::Version
                version: 0.0.0
            version:
        specification_version: 2
        summary: Legacy gem
        """);

    assertThat(spec.name(), is("legacy"));
    assertThat(spec.version(), is("0.9.1"));
    assertThat(spec.platform(), is("x86-mswin32-60"));
    assertThat(spec.authors(), contains("Old Author"));
    assertThat(spec.email(), contains("old@example.com"));
    assertThat(spec.licenses(), contains("GPL-2.0"));
    assertThat(spec.date(), is("2009-03-01"));
    assertThat(spec.dependencies(), contains(new GemDependency("json", "runtime", List.of("> 0.0.0"))));
    assertThat(spec.specificationVersion(), is(2));
    assertThat(spec.homepage(), is(nullValue()));
  }

  @Test
  void keepsVersionsThatLookLikeNumbers() {
    GemSpec spec = parse("""
        --- !ruby/object:Gem::Specification
        name: numbers
        version: !ruby/object:Gem::Version
          version: 1.10
        required_ruby_version: !ruby/object:Gem::Requirement
          requirements:
          - - "~>"
            - !ruby/object:Gem::Version
              version: 3.0
        """);

    assertThat(spec.version(), is("1.10"));
    assertThat(spec.requiredRubyVersion(), contains("~> 3.0"));
  }

  @Test
  void rejectsInvalidNamesAndVersions() {
    assertThrows(IllegalArgumentException.class, () -> parse("name: ../evil\nversion: 1.0\n"));
    assertThrows(IllegalArgumentException.class, () -> parse("name: good\nversion: 1.0/../2\n"));
    assertThrows(IllegalArgumentException.class, () -> parse("- not a spec\n"));
    assertThrows(IllegalArgumentException.class, () -> parse("name: good\nversion: 1.0\nplatform: a/b\n"));
  }
}
