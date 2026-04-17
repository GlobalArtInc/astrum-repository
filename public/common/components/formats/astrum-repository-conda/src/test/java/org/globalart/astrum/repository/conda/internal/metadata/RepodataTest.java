package org.globalart.astrum.repository.conda.internal.metadata;

import java.io.ByteArrayInputStream;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.internal.metadata.Repodata.PackageRecord;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.github.luben.zstd.ZstdInputStream;
import com.github.luben.zstd.ZstdOutputStream;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;

public class RepodataTest
{
  private static final ObjectMapper MAPPER = new ObjectMapper();

  @Test
  public void buildsRepodataFromRecords() throws Exception {
    byte[] json = Repodata.build("linux-64", List.of(
        new PackageRecord("b-1.0-0.tar.bz2", AssetKind.ARCH_TAR_PACKAGE, Map.of("name", "b", "md5", "x")),
        new PackageRecord("a-1.0-0.conda", AssetKind.ARCH_CONDA_PACKAGE, Map.of("name", "a", "size", 12))));
    JsonNode repodata = MAPPER.readTree(json);
    assertEquals("linux-64", repodata.path("info").path("subdir").asText());
    assertEquals("b", repodata.path("packages").path("b-1.0-0.tar.bz2").path("name").asText());
    assertEquals(12, repodata.path("packages.conda").path("a-1.0-0.conda").path("size").asInt());
    assertTrue(repodata.path("removed").isArray());
    assertEquals(1, repodata.path("repodata_version").asInt());
  }

  @Test
  public void mergesMembersWithFirstEntryWinning() throws Exception {
    String first = "{\"info\":{\"subdir\":\"noarch\"},"
        + "\"packages\":{\"a-1.0-0.tar.bz2\":{\"name\":\"a\",\"md5\":\"first\"}},"
        + "\"packages.conda\":{\"c-1.0-0.conda\":{\"name\":\"c\",\"depends\":[\"python >=3.8\",{\"x\":1}]}},"
        + "\"removed\":[\"old-1.0-0.tar.bz2\"],\"repodata_version\":1}";
    String second = "{\"info\":{\"subdir\":\"noarch\",\"base_url\":\"\"},"
        + "\"packages\":{\"a-1.0-0.tar.bz2\":{\"name\":\"a\",\"md5\":\"second\"},"
        + "\"b-2.0-1.tar.bz2\":{\"name\":\"b\"}},"
        + "\"packages.conda\":{},\"signatures\":{\"x\":{}},"
        + "\"removed\":[\"old-1.0-0.tar.bz2\",\"gone-2-0.conda\"],\"repodata_version\":2}";

    JsonNode merged;
    try (InputStream in = Repodata.merge("noarch", List.of(
        () -> new ByteArrayInputStream(first.getBytes(UTF_8)),
        () -> new ZstdInputStream(new ByteArrayInputStream(zstd(second)))))) {
      merged = MAPPER.readTree(in);
    }

    assertEquals("noarch", merged.path("info").path("subdir").asText());
    assertFalse(merged.path("info").has("base_url"));
    assertEquals(2, merged.path("packages").size());
    assertEquals("first", merged.path("packages").path("a-1.0-0.tar.bz2").path("md5").asText());
    assertEquals("b", merged.path("packages").path("b-2.0-1.tar.bz2").path("name").asText());
    assertEquals(1, merged.path("packages.conda").size());
    assertEquals(1, merged.path("packages.conda").path("c-1.0-0.conda").path("depends").get(1).path("x").asInt());
    assertEquals(2, merged.path("removed").size());
    assertFalse(merged.has("signatures"));
    assertEquals(1, merged.path("repodata_version").asInt());
  }

  @Test
  public void mergesEmptyMemberList() throws Exception {
    try (InputStream in = Repodata.merge("osx-arm64", List.of())) {
      JsonNode merged = MAPPER.readTree(in);
      assertEquals(0, merged.path("packages").size());
      assertEquals(0, merged.path("packages.conda").size());
    }
  }

  private static byte[] zstd(final String value) throws IOException {
    ByteArrayOutputStream out = new ByteArrayOutputStream();
    try (ZstdOutputStream zstd = new ZstdOutputStream(out)) {
      zstd.write(value.getBytes(UTF_8));
    }
    return out.toByteArray();
  }
}
