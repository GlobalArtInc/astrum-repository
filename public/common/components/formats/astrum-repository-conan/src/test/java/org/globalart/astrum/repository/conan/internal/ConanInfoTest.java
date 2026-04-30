package org.globalart.astrum.repository.conan.internal;

import java.time.OffsetDateTime;
import java.util.List;
import java.util.Map;

import org.junit.Test;

import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;

public class ConanInfoTest
{
  private static final String V2_INFO = """
      [settings]
      arch=armv8
      build_type=Release
      os=Macos
      [options]
      shared=True
      [requires]
      zlib/1.3.Z
      """;

  private static final String V1_INFO = """
      [settings]
          os=Linux
      [options]
          fPIC=True
      [full_requires]
          zlib/1.2.11:abc
      [recipe_hash]
          0123456789abcdef
      """;

  @Test
  public void parsesV2Sections() {
    ConanInfo info = ConanInfo.parse(V2_INFO);
    assertThat(info.values("settings"), is(equalTo(Map.of("arch", "armv8", "build_type", "Release", "os", "Macos"))));
    assertThat(info.values("options"), is(equalTo(Map.of("shared", "True"))));
    Map<String, Object> json = info.toV2Json();
    assertThat(json.get("requires"), is(equalTo(List.of("zlib/1.3.Z"))));
    assertThat(json.get("content"), is(equalTo(V2_INFO)));
  }

  @Test
  public void parsesV1Sections() {
    Map<String, Object> json = ConanInfo.parse(V1_INFO).toV1Json();
    assertThat(json.get("settings"), is(equalTo(Map.of("os", "Linux"))));
    assertThat(json.get("full_requires"), is(equalTo(List.of("zlib/1.2.11:abc"))));
    assertThat(json.get("recipe_hash"), is(equalTo("0123456789abcdef")));
  }

  @Test
  public void revisionTimeRoundTrip() {
    OffsetDateTime time = ConanRevision.parseTime("2025-12-09T12:51:39.337+0000");
    ConanRevision revision = new ConanRevision("cac0f6daea041b0ccf42934163defb20", time);
    assertThat(revision.isoTime(), is(equalTo("2025-12-09T12:51:39.337+0000")));
    assertThat(ConanRevision.fromJson(revision.toJson()), is(equalTo(revision)));
    assertThat(ConanRevision.parseTime("2024-01-02T03:04:05Z").getYear(), is(2024));
  }
}
