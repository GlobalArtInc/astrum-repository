package org.globalart.astrum.repository.terraform.rest;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.model.TerraformAttributes;
import org.globalart.astrum.rest.ValidationErrorsException;

import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class TerraformRestAttributesTest
{
  private final RepositoryManager repositoryManager = mock(RepositoryManager.class);

  @Test
  public void storesTheTerraformSettings() {
    when(repositoryManager.browse()).thenReturn(List.of());
    Configuration configuration = configuration("tf-group");

    TerraformRestAttributes.apply(configuration,
        new TerraformAttributes(true, "Terraform.Example.com", "registry.terraform.io"), repositoryManager);

    Map<String, Object> terraform = configuration.getAttributes().get("terraform");
    assertThat(terraform).containsEntry("requireAuthentication", true)
        .containsEntry("registryHost", "terraform.example.com")
        .containsEntry("upstreamHost", "registry.terraform.io");
  }

  @Test
  public void clearsHostsThatAreLeftEmpty() {
    when(repositoryManager.browse()).thenReturn(List.of());
    Configuration configuration = configuration("tf-group");
    configuration.attributes("terraform").set("registryHost", "old.example.com");

    TerraformRestAttributes.apply(configuration, null, repositoryManager);

    assertThat(configuration.getAttributes().get("terraform")).doesNotContainKey("registryHost")
        .containsEntry("requireAuthentication", false);
  }

  @Test
  public void rejectsInvalidAndTakenHosts() {
    Repository other = mock(Repository.class);
    when(other.getName()).thenReturn("tf-other");
    when(other.getFormat()).thenReturn(new Format("terraform") { });
    Configuration otherConfiguration = configuration("tf-other");
    otherConfiguration.attributes("terraform").set("registryHost", "terraform.example.com");
    when(other.getConfiguration()).thenReturn(otherConfiguration);
    when(repositoryManager.browse()).thenReturn(List.of(other));

    assertThatThrownBy(() -> TerraformRestAttributes.apply(configuration("tf-group"),
        new TerraformAttributes(false, "terraform.example.com", null), repositoryManager))
        .isInstanceOf(ValidationErrorsException.class);
    assertThatThrownBy(() -> TerraformRestAttributes.apply(configuration("tf-group"),
        new TerraformAttributes(false, "https://terraform.example.com/", null), repositoryManager))
        .isInstanceOf(ValidationErrorsException.class);
    assertThatThrownBy(() -> TerraformRestAttributes.apply(configuration("tf-group"),
        new TerraformAttributes(false, null, "not a host"), repositoryManager))
        .isInstanceOf(ValidationErrorsException.class);
  }

  private static Configuration configuration(final String name) {
    Configuration configuration = new ConfigurationData();
    configuration.setRepositoryName(name);
    configuration.setAttributes(new HashMap<>());
    return configuration;
  }
}
