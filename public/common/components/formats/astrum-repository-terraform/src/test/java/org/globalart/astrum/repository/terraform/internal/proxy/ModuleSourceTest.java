package org.globalart.astrum.repository.terraform.internal.proxy;

import java.net.URI;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

public class ModuleSourceTest
{
  @Test
  public void cachesArchivesOverHttp() {
    ModuleSource source = ModuleSource.parse(
        "https://api.github.com/repos/hashicorp/terraform-aws-consul/tarball/v0.0.1//*?archive=tar.gz");

    assertThat(source.cacheable()).isTrue();
    assertThat(source.archiveUrl())
        .isEqualTo("https://api.github.com/repos/hashicorp/terraform-aws-consul/tarball/v0.0.1");
    assertThat(source.extension()).isEqualTo("tar.gz");
    assertThat(source.subdir()).isEqualTo("*");
    assertThat(source.subdirSuffix()).isEqualTo("//*");
  }

  @Test
  public void keepsTheQueryOfTheArchiveUrlButNotTheArchiveArgument() {
    ModuleSource source = ModuleSource.parse("https://bucket.example.com/vpc.zip?X-Amz-Signature=abc&archive=zip");

    assertThat(source.archiveUrl()).isEqualTo("https://bucket.example.com/vpc.zip?X-Amz-Signature=abc");
    assertThat(source.extension()).isEqualTo("zip");
    assertThat(source.subdir()).isNull();
  }

  @Test
  public void turnsGitHubTagsIntoTarballs() {
    ModuleSource source = ModuleSource.parse(
        "git::https://github.com/terraform-aws-modules/terraform-aws-vpc?ref=v5.0.0");

    assertThat(source.archiveUrl())
        .isEqualTo("https://codeload.github.com/terraform-aws-modules/terraform-aws-vpc/tar.gz/v5.0.0");
    assertThat(source.subdir()).isEqualTo("*");

    ModuleSource nested = ModuleSource.parse("git::https://github.com/acme/modules.git//network/vpc?ref=abc123");
    assertThat(nested.archiveUrl()).isEqualTo("https://codeload.github.com/acme/modules/tar.gz/abc123");
    assertThat(nested.subdir()).isEqualTo("*/network/vpc");

    assertThat(ModuleSource.parse("github.com/acme/modules?ref=v1").archiveUrl())
        .isEqualTo("https://codeload.github.com/acme/modules/tar.gz/v1");
  }

  @Test
  public void handsOtherLocationsToTerraformUnchanged() {
    assertThat(ModuleSource.parse("git::ssh://git@example.com/vpc.git?ref=v1").cacheable()).isFalse();
    assertThat(ModuleSource.parse("git::https://gitlab.com/acme/vpc.git?ref=v1").cacheable()).isFalse();
    assertThat(ModuleSource.parse("s3::https://s3.amazonaws.com/bucket/vpc.zip").cacheable()).isFalse();
    assertThat(ModuleSource.parse("https://example.com/vpc").cacheable()).isFalse();
    assertThat(ModuleSource.parse("git::https://github.com/acme/vpc?ref=v1&sshkey=abc").cacheable()).isFalse();
  }

  @Test
  public void resolvesLocationsRelativeToTheDownloadEndpoint() {
    URI endpoint = URI.create("https://registry.example.com/v1/modules/acme/vpc/aws/1.0.0/download");

    assertThat(ModuleSource.resolve(endpoint, "./vpc.zip"))
        .isEqualTo("https://registry.example.com/v1/modules/acme/vpc/aws/1.0.0/vpc.zip");
    assertThat(ModuleSource.resolve(endpoint, "/archives/vpc.tar.gz//*?archive=tar.gz"))
        .isEqualTo("https://registry.example.com/archives/vpc.tar.gz//*?archive=tar.gz");
    assertThat(ModuleSource.resolve(endpoint, "../../../../../../archives/vpc.zip"))
        .isEqualTo("https://registry.example.com/archives/vpc.zip");
    assertThat(ModuleSource.resolve(endpoint, "git::https://github.com/acme/vpc"))
        .isEqualTo("git::https://github.com/acme/vpc");
  }
}
