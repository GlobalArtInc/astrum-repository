package org.globalart.astrum.repository.terraform;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Matcher;
import org.globalart.astrum.repository.view.Request;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;

public class TerraformPathsTest
{
  @Test
  public void validatesAddressParts() {
    assertThat(TerraformPaths.isValidName("terraform-aws-modules")).isTrue();
    assertThat(TerraformPaths.isValidName("my_module")).isTrue();
    assertThat(TerraformPaths.isValidName("-bad")).isFalse();
    assertThat(TerraformPaths.isValidName("..")).isFalse();
    assertThat(TerraformPaths.isValidVersion("1.2.3")).isTrue();
    assertThat(TerraformPaths.isValidVersion("1.2.3-beta.1+build.5")).isTrue();
    assertThat(TerraformPaths.isValidVersion("v1.2.3")).isFalse();
    assertThat(TerraformPaths.isValidVersion("1.2")).isFalse();
    assertThat(TerraformPaths.isValidHostname("registry.terraform.io")).isTrue();
    assertThat(TerraformPaths.isValidHostname("localhost:8443")).isTrue();
    assertThat(TerraformPaths.isValidHostname("bad/host")).isFalse();
  }

  @Test
  public void computesRelativeUrlsThatResolveAgainstTheDocument() {
    assertThat(TerraformPaths.relative("/.well-known/terraform.json", "/v1/modules/")).isEqualTo("../v1/modules/");
    assertThat(TerraformPaths.relative("/v1/modules/acme/vpc/aws/1.0.0/download",
        "/v1/modules/acme/vpc/aws/1.0.0/vpc-aws-1.0.0.tar.gz")).isEqualTo("./vpc-aws-1.0.0.tar.gz");
    assertThat(TerraformPaths.relative("/v1/providers/acme/demo/1.0.0/download/linux/amd64",
        "/v1/providers/acme/demo/1.0.0/terraform-provider-demo_1.0.0_linux_amd64.zip"))
        .isEqualTo("../../terraform-provider-demo_1.0.0_linux_amd64.zip");
    assertThat(TerraformPaths.relative("/registry.terraform.io/hashicorp/random/3.6.3.json",
        "/v1/providers/hashicorp/random/3.6.3/terraform-provider-random_3.6.3_linux_amd64.zip"))
        .isEqualTo("../../../v1/providers/hashicorp/random/3.6.3/terraform-provider-random_3.6.3_linux_amd64.zip");
  }

  @Test
  public void routesModuleRequests() {
    Context versions = matched(TerraformPaths.moduleVersions(), "GET", "/v1/modules/acme/vpc/aws/versions");
    assertThat(TerraformRequest.of(versions).module()).isEqualTo(new ModuleCoordinates("acme", "vpc", "aws", null));
    assertThat(versions.getAttributes().get(AssetKind.class)).isEqualTo(AssetKind.MODULE_VERSIONS);

    Context download = matched(TerraformPaths.moduleDownload(), "GET", "/v1/modules/acme/vpc/aws/1.0.0/download");
    assertThat(TerraformRequest.of(download).module().version()).isEqualTo("1.0.0");

    Context archive = matched(TerraformPaths.moduleArchive("GET"), "GET",
        "/v1/modules/acme/vpc/aws/1.0.0/vpc-aws-1.0.0.tar.gz");
    assertThat(TerraformRequest.of(archive).assetPath())
        .isEqualTo("/v1/modules/acme/vpc/aws/1.0.0/vpc-aws-1.0.0.tar.gz");

    assertThat(matches(TerraformPaths.moduleArchive("GET"), "GET", "/v1/modules/acme/vpc/aws/1.0.0/download"))
        .isFalse();
    assertThat(matches(TerraformPaths.moduleArchive("GET"), "GET", "/v1/modules/acme/vpc/aws/1.0.0/notes.txt"))
        .isFalse();
    assertThat(matches(TerraformPaths.moduleVersions(), "PUT", "/v1/modules/acme/vpc/aws/versions")).isFalse();
    assertThat(matches(TerraformPaths.moduleDownload(), "GET", "/v1/modules/acme/vpc/aws/latest/download"))
        .isFalse();
  }

  @Test
  public void routesProviderRequests() {
    Context download = matched(TerraformPaths.providerDownload(), "GET",
        "/v1/providers/acme/demo/1.0.0/download/linux/amd64");
    TerraformRequest request = TerraformRequest.of(download);
    assertThat(request.provider()).isEqualTo(new ProviderCoordinates("acme", "demo", "1.0.0"));
    assertThat(request.platform()).isEqualTo(new Platform("linux", "amd64"));
    assertThat(request.assetPath()).isEqualTo("/v1/providers/acme/demo/1.0.0/download/linux/amd64");

    Context file = matched(TerraformPaths.providerFile("GET"), "GET",
        "/v1/providers/acme/demo/1.0.0/terraform-provider-demo_1.0.0_SHA256SUMS.72D7468F.sig");
    assertThat(file.getAttributes().get(AssetKind.class)).isEqualTo(AssetKind.PROVIDER_SIGNATURE);

    Context zip = matched(TerraformPaths.providerFile("GET"), "GET",
        "/v1/providers/acme/demo/1.0.0/terraform-provider-demo_1.0.0_darwin_arm64.zip");
    assertThat(TerraformRequest.of(zip).platform()).isEqualTo(new Platform("darwin", "arm64"));

    assertThat(matches(TerraformPaths.providerFile("GET"), "GET",
        "/v1/providers/acme/demo/1.0.0/terraform-provider-other_1.0.0_linux_amd64.zip")).isFalse();
  }

  @Test
  public void routesNetworkMirrorRequests() {
    Context index = matched(TerraformPaths.mirrorIndex(), "GET", "/Registry.Terraform.io/hashicorp/random/index.json");
    assertThat(TerraformRequest.of(index).hostname()).isEqualTo("registry.terraform.io");

    Context version = matched(TerraformPaths.mirrorVersion(), "GET",
        "/registry.terraform.io/hashicorp/random/3.6.3.json");
    assertThat(TerraformRequest.of(version).provider().version()).isEqualTo("3.6.3");

    assertThat(matches(TerraformPaths.mirrorVersion(), "GET", "/registry.terraform.io/hashicorp/random/index.json"))
        .isFalse();
    assertThat(matches(TerraformPaths.mirrorIndex(), "GET", "/v1/providers/hashicorp/random/versions")).isFalse();
  }

  private static Context matched(final Matcher matcher, final String action, final String path) {
    Context context = context(action, path);
    assertThat(matcher.matches(context)).as(path).isTrue();
    return context;
  }

  private static boolean matches(final Matcher matcher, final String action, final String path) {
    return matcher.matches(context(action, path));
  }

  private static Context context(final String action, final String path) {
    return new Context(mock(Repository.class), new Request.Builder().action(action).path(path).build());
  }
}
