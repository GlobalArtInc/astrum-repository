package org.globalart.astrum.repository.terraform;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

public class ProviderFilesTest
{
  private final ProviderCoordinates provider = new ProviderCoordinates("acme", "demo", "1.2.0");

  @Test
  public void recognizesReleaseFilesByName() {
    assertThat(ProviderFiles.kindOf(provider, "terraform-provider-demo_1.2.0_linux_amd64.zip"))
        .contains(AssetKind.PROVIDER_ARCHIVE);
    assertThat(ProviderFiles.kindOf(provider, "terraform-provider-demo_1.2.0_SHA256SUMS"))
        .contains(AssetKind.PROVIDER_SHASUMS);
    assertThat(ProviderFiles.kindOf(provider, "terraform-provider-demo_1.2.0_SHA256SUMS.sig"))
        .contains(AssetKind.PROVIDER_SIGNATURE);
    assertThat(ProviderFiles.kindOf(provider, "terraform-provider-demo_1.2.0_SHA256SUMS.72D7468F.sig"))
        .contains(AssetKind.PROVIDER_SIGNATURE);
    assertThat(ProviderFiles.kindOf(provider, "terraform-provider-demo_1.2.0_manifest.json"))
        .contains(AssetKind.PROVIDER_MANIFEST);
    assertThat(ProviderFiles.kindOf(provider, "signing-key.asc")).contains(AssetKind.PROVIDER_KEY);
    assertThat(ProviderFiles.kindOf(provider, "terraform-provider-demo_1.3.0_linux_amd64.zip")).isEmpty();
    assertThat(ProviderFiles.kindOf(provider, "terraform-provider-demo_1.2.0_linux.zip")).isEmpty();
    assertThat(ProviderFiles.kindOf(provider, "README.md")).isEmpty();
  }

  @Test
  public void readsThePlatformOfAPackage() {
    assertThat(ProviderFiles.platformOf(provider, "terraform-provider-demo_1.2.0_windows_386.zip"))
        .contains(new Platform("windows", "386"));
    assertThat(ProviderFiles.platformOf(provider, "terraform-provider-demo_1.2.0_linux_arm_64.zip")).isEmpty();
  }

  @Test
  public void namesTheReleaseFilesOfAVersion() {
    assertThat(provider.archiveFileName(new Platform("linux", "amd64")))
        .isEqualTo("terraform-provider-demo_1.2.0_linux_amd64.zip");
    assertThat(provider.filePath(provider.signatureFileName()))
        .isEqualTo("/v1/providers/acme/demo/1.2.0/terraform-provider-demo_1.2.0_SHA256SUMS.sig");
    assertThat(ProviderFiles.isKeyUpload("release-key.ASC")).isTrue();
    assertThat(ProviderFiles.isKeyUpload("notes.txt")).isFalse();
  }
}
