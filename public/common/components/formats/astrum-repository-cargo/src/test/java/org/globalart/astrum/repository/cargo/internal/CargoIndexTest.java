package org.globalart.astrum.repository.cargo.internal;

import java.time.Instant;
import java.util.List;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.CoreMatchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;

public class CargoIndexTest
{
  private static final String METADATA = "{"
      + "\"name\":\"foo\",\"vers\":\"0.1.0\","
      + "\"deps\":["
      + "{\"name\":\"rand\",\"version_req\":\"^0.8\",\"features\":[\"small_rng\"],\"optional\":true,"
      + "\"default_features\":false,\"target\":null,\"kind\":\"normal\","
      + "\"registry\":\"https://github.com/rust-lang/crates.io-index\",\"explicit_name_in_toml\":\"random\"},"
      + "{\"name\":\"bar\",\"version_req\":\"^1\",\"features\":[],\"optional\":false,\"default_features\":true,"
      + "\"target\":\"cfg(unix)\",\"kind\":\"dev\"}],"
      + "\"features\":{\"default\":[\"std\"],\"std\":[],\"fast\":[\"dep:random\",\"bar?/fast\"]},"
      + "\"links\":\"foo\",\"rust_version\":\"1.70\",\"description\":\"A crate\"}";

  @Test
  public void buildsIndexEntry() {
    JsonNode metadata = CargoResponses.read(METADATA.getBytes(UTF_8));
    ObjectNode entry = CargoIndex.entry(metadata, "abc123", Instant.parse("2026-10-09T12:00:00.5Z"));

    assertThat(entry.get("name").asText(), is(equalTo("foo")));
    assertThat(entry.get("vers").asText(), is(equalTo("0.1.0")));
    assertThat(entry.get("cksum").asText(), is(equalTo("abc123")));
    assertThat(entry.get("yanked").asBoolean(), is(false));
    assertThat(entry.get("links").asText(), is(equalTo("foo")));
    assertThat(entry.get("rust_version").asText(), is(equalTo("1.70")));
    assertThat(entry.get("pubtime").asText(), is(equalTo("2026-10-09T12:00:00Z")));
    assertThat(entry.get("v").asInt(), is(2));
    assertThat(entry.get("features").has("default"), is(true));
    assertThat(entry.get("features").has("fast"), is(false));
    assertThat(entry.get("features2").get("fast").size(), is(2));
    assertThat(entry.has("description"), is(false));

    JsonNode renamed = entry.get("deps").get(0);
    assertThat(renamed.get("name").asText(), is(equalTo("random")));
    assertThat(renamed.get("package").asText(), is(equalTo("rand")));
    assertThat(renamed.get("req").asText(), is(equalTo("^0.8")));
    assertThat(renamed.get("optional").asBoolean(), is(true));
    assertThat(renamed.get("default_features").asBoolean(), is(false));
    assertThat(renamed.get("target").isNull(), is(true));
    assertThat(renamed.get("registry").asText(), is(equalTo("https://github.com/rust-lang/crates.io-index")));

    JsonNode plain = entry.get("deps").get(1);
    assertThat(plain.get("name").asText(), is(equalTo("bar")));
    assertThat(plain.has("package"), is(false));
    assertThat(plain.has("registry"), is(false));
    assertThat(plain.get("kind").asText(), is(equalTo("dev")));
    assertThat(plain.get("target").asText(), is(equalTo("cfg(unix)")));
  }

  @Test
  public void omitsOptionalFields() {
    JsonNode metadata = CargoResponses.read("{\"name\":\"a\",\"vers\":\"1.0.0\",\"features\":{\"x\":[\"a/b\"]}}"
        .getBytes(UTF_8));
    ObjectNode entry = CargoIndex.entry(metadata, "00", Instant.EPOCH);
    assertThat(entry.has("v"), is(false));
    assertThat(entry.has("features2"), is(false));
    assertThat(entry.has("links"), is(false));
    assertThat(entry.has("rust_version"), is(false));
    assertThat(entry.get("deps").size(), is(0));
  }

  @Test
  public void yanksEntries() {
    String line = "{\"name\":\"a\",\"vers\":\"1.0.0\",\"yanked\":false}";
    assertThat(CargoIndex.withYanked(line, true), is(equalTo("{\"name\":\"a\",\"vers\":\"1.0.0\",\"yanked\":true}")));
  }

  @Test
  public void mergesFilesKeepingFirstVersion() {
    byte[] hosted = ("{\"name\":\"a\",\"vers\":\"1.0.0\",\"cksum\":\"hosted\"}\n"
        + "{\"name\":\"a\",\"vers\":\"2.0.0\",\"cksum\":\"hosted\"}\n").getBytes(UTF_8);
    byte[] proxy = ("{\"name\":\"a\",\"vers\":\"1.0.0\",\"cksum\":\"proxy\"}\n\n"
        + "{\"name\":\"a\",\"vers\":\"0.9.0\",\"cksum\":\"proxy\"}\nnot json\n").getBytes(UTF_8);
    List<String> lines = CargoIndex.lines(CargoIndex.merge(List.of(hosted, proxy)));
    assertThat(lines.size(), is(3));
    assertThat(CargoIndex.version(lines.get(0)), is(equalTo("1.0.0")));
    assertThat(lines.get(0).contains("hosted"), is(true));
    assertThat(CargoIndex.version(lines.get(2)), is(equalTo("0.9.0")));
    assertThat(CargoIndex.version("not json"), is(nullValue()));
  }
}
