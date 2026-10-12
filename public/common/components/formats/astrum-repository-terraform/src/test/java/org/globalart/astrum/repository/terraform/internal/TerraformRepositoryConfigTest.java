package org.globalart.astrum.repository.terraform.internal;

import java.util.Map;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.internal.ConfigurationData;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

public class TerraformRepositoryConfigTest
{
  @Test
  public void normalizesRegistryHosts() {
    assertThat(TerraformRepositoryConfig.normalizeHost(" Terraform.Example.COM ")).contains("terraform.example.com");
    assertThat(TerraformRepositoryConfig.normalizeHost("astrum.example.com:443")).contains("astrum.example.com");
    assertThat(TerraformRepositoryConfig.normalizeHost("astrum.example.com:8443")).contains("astrum.example.com:8443");
    assertThat(TerraformRepositoryConfig.normalizeHost("https://astrum.example.com")).isEmpty();
    assertThat(TerraformRepositoryConfig.normalizeHost("")).isEmpty();
  }

  @Test
  public void matchesTheHostOfARequest() {
    assertThat(TerraformRepositoryConfig.matchesHost("astrum.example.com", "Astrum.Example.com", 8081)).isTrue();
    assertThat(TerraformRepositoryConfig.matchesHost("astrum.example.com:8443", "astrum.example.com", 8443)).isTrue();
    assertThat(TerraformRepositoryConfig.matchesHost("astrum.example.com:8443", "astrum.example.com", 443)).isFalse();
    assertThat(TerraformRepositoryConfig.matchesHost("astrum.example.com", "other.example.com", 443)).isFalse();
  }

  @Test
  public void readsTheTerraformAttributes() {
    Configuration configuration = new ConfigurationData();
    configuration.setAttributes(Map.of("terraform", Map.of("requireAuthentication", true,
        "registryHost", "Terraform.Example.com", "upstreamHost", "registry.terraform.io")));

    assertThat(TerraformRepositoryConfig.requireAuthentication(configuration)).isTrue();
    assertThat(TerraformRepositoryConfig.registryHost(configuration)).contains("terraform.example.com");
    assertThat(TerraformRepositoryConfig.upstreamHost(configuration)).contains("registry.terraform.io");
    assertThat(TerraformRepositoryConfig.requireAuthentication((Configuration) null)).isFalse();
    assertThat(TerraformRepositoryConfig.registryHost(new ConfigurationData())).isEmpty();
  }
}
