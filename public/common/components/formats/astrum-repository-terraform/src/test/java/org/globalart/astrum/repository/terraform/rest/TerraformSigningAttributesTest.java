package org.globalart.astrum.repository.terraform.rest;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.internal.ConfigurationData;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class TerraformSigningAttributesTest
{
  @Test
  public void keepsTheSavedPassphraseWhileTheKeyStaysTheSame() {
    Repository existing = repository(Map.of("keypair", "KEY", "passphrase", "secret"));
    Configuration configuration = configuration();

    TerraformSigningAttributes.apply(configuration, new TerraformSigningAttributes(" KEY ", null), existing);

    assertThat(configuration.getAttributes().get("terraformSigning"))
        .containsEntry("keypair", "KEY").containsEntry("passphrase", "secret");
  }

  @Test
  public void dropsTheSavedPassphraseWithANewKey() {
    Repository existing = repository(Map.of("keypair", "KEY", "passphrase", "secret"));
    Configuration configuration = configuration();

    TerraformSigningAttributes.apply(configuration, new TerraformSigningAttributes("OTHER", ""), existing);

    assertThat(configuration.getAttributes().get("terraformSigning"))
        .containsEntry("keypair", "OTHER").doesNotContainKey("passphrase");
  }

  @Test
  public void neverReturnsThePassphrase() {
    TerraformSigningAttributes read = TerraformSigningAttributes.of(
        repository(Map.of("keypair", "KEY", "passphrase", "secret")));

    assertThat(read.getKeypair()).isEqualTo("KEY");
    assertThat(read.getPassphrase()).isNull();
    assertThat(TerraformSigningAttributes.of(repository(Map.of()))).isNull();
  }

  private static Repository repository(final Map<String, Object> signing) {
    Configuration configuration = configuration();
    configuration.getAttributes().put("terraformSigning", new HashMap<>(signing));
    Repository repository = mock(Repository.class);
    when(repository.getConfiguration()).thenReturn(configuration);
    return repository;
  }

  private static Configuration configuration() {
    Configuration configuration = new ConfigurationData();
    configuration.setAttributes(new HashMap<>());
    return configuration;
  }
}
