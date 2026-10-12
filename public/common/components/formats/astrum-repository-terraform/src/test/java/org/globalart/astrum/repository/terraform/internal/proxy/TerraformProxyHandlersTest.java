package org.globalart.astrum.repository.terraform.internal.proxy;

import java.net.URI;
import java.time.Clock;

import org.globalart.astrum.repository.terraform.ModuleCoordinates;
import org.globalart.astrum.repository.terraform.Platform;
import org.globalart.astrum.repository.terraform.ProviderCoordinates;
import org.globalart.astrum.repository.terraform.internal.DownloadTokens;
import org.globalart.astrum.repository.terraform.internal.TerraformResponses;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.junit.Test;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.assertj.core.api.Assertions.assertThat;

public class TerraformProxyHandlersTest
{
  private final DownloadTokens tokens = new DownloadTokens("secret".getBytes(UTF_8), Clock.systemUTC());

  @Test
  public void movesReleaseFilesIntoTheRepository() throws Exception {
    ProviderCoordinates provider = new ProviderCoordinates("hashicorp", "random", "3.6.3");
    Platform platform = new Platform("linux", "amd64");
    URI endpoint = URI.create("https://registry.terraform.io/v1/providers/hashicorp/random/3.6.3/download/linux/amd64");
    JsonNode upstream = TerraformResponses.read(("{\"os\":\"linux\",\"arch\":\"amd64\","
        + "\"filename\":\"terraform-provider-random_3.6.3_linux_amd64.zip\","
        + "\"download_url\":\"https://releases.hashicorp.com/terraform-provider-random/3.6.3/"
        + "terraform-provider-random_3.6.3_linux_amd64.zip\","
        + "\"shasums_url\":\"https://releases.hashicorp.com/terraform-provider-random/3.6.3/"
        + "terraform-provider-random_3.6.3_SHA256SUMS\","
        + "\"shasums_signature_url\":\"/sums/random.sig\","
        + "\"shasum\":\"abc\",\"signing_keys\":{\"gpg_public_keys\":[]}}").getBytes(UTF_8));

    ObjectNode rewritten = TerraformProxyHandlers.rewrite(upstream, provider, platform, endpoint, tokens);

    assertThat(rewritten.path("download_url").asText())
        .startsWith("../../terraform-provider-random_3.6.3_linux_amd64.zip?expires=");
    assertThat(rewritten.path("shasums_url").asText())
        .startsWith("../../terraform-provider-random_3.6.3_SHA256SUMS?expires=");
    assertThat(rewritten.path("shasums_signature_url").asText())
        .isEqualTo("https://registry.terraform.io/sums/random.sig");
    assertThat(rewritten.path("shasum").asText()).isEqualTo("abc");
    assertThat(upstream.path("download_url").asText()).startsWith("https://releases.hashicorp.com/");
  }

  @Test
  public void pointsTerraformAtTheCachedModulePackage() {
    ModuleCoordinates module = new ModuleCoordinates("terraform-aws-modules", "vpc", "aws", "5.0.0");

    assertThat(TerraformProxyHandlers.localLocation(module,
        "https://example.com/vpc.tar.gz//*?archive=tar.gz", tokens))
        .startsWith("./vpc-aws-5.0.0.tar.gz//*?expires=");
    assertThat(TerraformProxyHandlers.localLocation(module, "git::ssh://git@example.com/vpc.git", tokens))
        .isEqualTo("git::ssh://git@example.com/vpc.git");
  }

  @Test
  public void readsTheFileNameOfAUrl() {
    assertThat(TerraformProxyFacet.baseName("https://releases.example.com/a/b/file.zip?token=1")).isEqualTo("file.zip");
    assertThat(TerraformProxyFacet.baseName("../../file_SHA256SUMS")).isEqualTo("file_SHA256SUMS");
  }
}
