package org.globalart.astrum.repository.cargo.internal;

import java.util.List;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.hamcrest.CoreMatchers.equalTo;
import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;

public class CargoSearchTest
{
  @Test
  public void mergesResultsOfMembers() {
    JsonNode hosted = CargoResponses.read(("{\"crates\":[{\"name\":\"serde-ext\",\"max_version\":\"0.2.0\"}],"
        + "\"meta\":{\"total\":1}}").getBytes(UTF_8));
    JsonNode proxy = CargoResponses.read(("{\"crates\":[{\"name\":\"serde\",\"max_version\":\"1.0.0\"},"
        + "{\"name\":\"serde-ext\",\"max_version\":\"0.1.0\"}],\"meta\":{\"total\":250}}").getBytes(UTF_8));

    ObjectNode merged = CargoSearch.merge(List.of(hosted, proxy), 10);

    assertThat(merged.get("crates").size(), is(2));
    assertThat(merged.get("crates").get(0).get("max_version").asText(), is(equalTo("0.2.0")));
    assertThat(merged.get("crates").get(1).get("name").asText(), is(equalTo("serde")));
    assertThat(merged.get("meta").get("total").asLong(), is(250L));

    assertThat(CargoSearch.merge(List.of(hosted, proxy), 1).get("crates").size(), is(1));
  }
}
