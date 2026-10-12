package org.globalart.astrum.repository.rubygems.internal.index;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.util.HexFormat;
import java.util.List;
import java.util.zip.GZIPInputStream;

import org.globalart.astrum.repository.rubygems.internal.index.LegacySpecs.Kind;

import org.junit.jupiter.api.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;

class LegacySpecsTest
{
  private static final List<String> VERSIONS = List.of("1.0.0", "1.2.0", "1.10.0.rc1", "1.1.0-java", "0.9-java");

  @Test
  void selectsReleasesPrereleasesAndLatest() {
    assertThat(LegacySpecs.select(Kind.SPECS, VERSIONS), contains("1.0.0", "1.2.0", "1.1.0-java", "0.9-java"));
    assertThat(LegacySpecs.select(Kind.PRERELEASE_SPECS, VERSIONS), contains("1.10.0.rc1"));
    assertThat(LegacySpecs.select(Kind.LATEST_SPECS, VERSIONS), containsInAnyOrder("1.2.0", "1.1.0-java"));
  }

  @Test
  void writesGzippedMarshalTuples() throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    LegacySpecs.write(Kind.SPECS, consumer -> consumer.accept("rake", List.of("13.0.1")), out);

    byte[] marshal = new GZIPInputStream(new ByteArrayInputStream(out.toByteArray())).readAllBytes();
    assertThat(HexFormat.of().formatHex(marshal), is("04085b065b0849220972616b65063a064554553a1147656d3a3a5665727369"
        + "6f6e5b0649220b31332e302e31063b005449220972756279063b0054"));
  }

  @Test
  void mapsPaths() {
    assertThat(Kind.fromPath("/latest_specs.4.8.gz"), is(Kind.LATEST_SPECS));
    assertThat(Kind.fromPath("/specs.4.8"), is(nullValue()));
  }
}
