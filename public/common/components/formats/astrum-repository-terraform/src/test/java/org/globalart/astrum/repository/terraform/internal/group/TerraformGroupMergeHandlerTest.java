package org.globalart.astrum.repository.terraform.internal.group;

import java.util.List;

import org.globalart.astrum.repository.terraform.internal.TerraformResponses;

import com.fasterxml.jackson.databind.JsonNode;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.assertj.core.api.Assertions.assertThat;

public class TerraformGroupMergeHandlerTest
{
  @Test
  public void mergesModuleVersionsInVersionOrder() throws Exception {
    JsonNode merged = TerraformGroupMergeHandler.mergeModuleVersions(List.of(
        json("{\"modules\":[{\"versions\":[{\"version\":\"1.10.0\"},{\"version\":\"1.2.0\"}]}]}"),
        json("{\"modules\":[{\"source\":\"acme/vpc/aws\",\"versions\":[{\"version\":\"1.2.0\"},"
            + "{\"version\":\"0.9.0\"}]}]}")));

    assertThat(merged.toString()).isEqualTo("{\"modules\":[{\"versions\":[{\"version\":\"0.9.0\"},"
        + "{\"version\":\"1.2.0\"},{\"version\":\"1.10.0\"}]}]}");
  }

  @Test
  public void mergesProviderVersionsWithTheFirstMemberWinning() throws Exception {
    JsonNode merged = TerraformGroupMergeHandler.mergeProviderVersions(List.of(
        json("{\"versions\":[{\"version\":\"2.0.0\",\"platforms\":[{\"os\":\"linux\",\"arch\":\"amd64\"}]}]}"),
        json("{\"versions\":[{\"version\":\"2.0.0\",\"platforms\":[]},{\"version\":\"1.0.0\"}]}")));

    assertThat(merged.path("versions")).hasSize(2);
    assertThat(merged.path("versions").path(0).path("version").asText()).isEqualTo("1.0.0");
    assertThat(merged.path("versions").path(1).path("platforms")).hasSize(1);
  }

  @Test
  public void mergesMirrorDocuments() throws Exception {
    JsonNode index = TerraformGroupMergeHandler.mergeMirrorIndexes(List.of(
        json("{\"versions\":{\"3.2.3\":{}}}"), json("{\"versions\":{\"3.2.4\":{},\"3.2.3\":{}}}")));
    assertThat(index.toString()).isEqualTo("{\"versions\":{\"3.2.3\":{},\"3.2.4\":{}}}");

    JsonNode version = TerraformGroupMergeHandler.mergeMirrorVersions(List.of(
        json("{\"archives\":{\"linux_amd64\":{\"url\":\"a\"}}}"),
        json("{\"archives\":{\"linux_amd64\":{\"url\":\"b\"},\"darwin_arm64\":{\"url\":\"c\"}}}")));
    assertThat(version.path("archives").path("linux_amd64").path("url").asText()).isEqualTo("a");
    assertThat(version.path("archives").path("darwin_arm64").path("url").asText()).isEqualTo("c");
  }

  private static JsonNode json(final String text) throws Exception {
    return TerraformResponses.read(text.getBytes(UTF_8));
  }
}
