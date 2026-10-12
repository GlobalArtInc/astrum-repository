package org.globalart.astrum.repository.terraform.internal.hosted;

import org.globalart.astrum.repository.terraform.ProviderCoordinates;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

public class TerraformHostedFacetTest
{
  @Test
  public void findsTheProviderVersionOfADeletedPackage() {
    assertThat(TerraformHostedFacet.versionDirectory(
        "v1/providers/acme/demo/1.0.0/terraform-provider-demo_1.0.0_linux_amd64.zip"))
        .contains(new ProviderCoordinates("acme", "demo", "1.0.0"));
    assertThat(TerraformHostedFacet.versionDirectory(
        "/v1/providers/acme/demo/1.0.0/terraform-provider-demo_1.0.0_SHA256SUMS")).isEmpty();
    assertThat(TerraformHostedFacet.versionDirectory("/v1/modules/acme/vpc/aws/1.0.0/vpc-aws-1.0.0.zip")).isEmpty();
  }
}
