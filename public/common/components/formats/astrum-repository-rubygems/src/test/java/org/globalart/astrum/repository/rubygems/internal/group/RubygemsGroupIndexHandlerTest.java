package org.globalart.astrum.repository.rubygems.internal.group;

import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.rubygems.internal.index.VersionsLine;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;

class RubygemsGroupIndexHandlerTest
{
  @Test
  void collectsAppendedAndYankedVersions() {
    Map<String, String> byName = new LinkedHashMap<>();
    RubygemsGroupIndexHandler.collect(byName, new VersionsLine("rack", List.of("1.0", "1.1", "2.0-java"), "a"));
    RubygemsGroupIndexHandler.collect(byName, new VersionsLine("rake", List.of("13.0"), "b"));
    RubygemsGroupIndexHandler.collect(byName, new VersionsLine("rack", List.of("2.1", "-1.1"), "c"));
    RubygemsGroupIndexHandler.collect(byName, new VersionsLine("rake", List.of("-13.0"), "d"));

    assertThat(byName, is(Map.of("rack", "1.0,2.0-java,2.1")));
  }
}
