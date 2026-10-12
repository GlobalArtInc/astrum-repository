package org.globalart.astrum.repository.yum.rest;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.rest.api.model.HostedStorageAttributes;
import org.globalart.astrum.repository.yum.internal.LayoutPolicy;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class YumRepositoryApiRequestToConfigurationConverterTest
{
  private RepositoryManager repositoryManager;

  private YumHostedRepositoryApiRequestToConfigurationConverter underTest;

  @Before
  public void setUp() {
    ConfigurationStore configurationStore = mock(ConfigurationStore.class);
    when(configurationStore.newConfiguration()).thenAnswer(invocation -> new ConfigurationData());
    repositoryManager = mock(RepositoryManager.class);
    underTest = new YumHostedRepositoryApiRequestToConfigurationConverter(repositoryManager);
    underTest.setConfigurationStore(configurationStore);
  }

  @Test
  public void writesTheRepodataDepthAndLayoutPolicy() {
    Configuration configuration = underTest.convert(request(new YumAttributes(3, LayoutPolicy.PERMISSIVE), null));

    assertEquals("yum-hosted", configuration.getRecipeName());
    assertEquals(Integer.valueOf(3), configuration.attributes("yum").get("repodataDepth", Integer.class));
    assertEquals("PERMISSIVE", configuration.attributes("yum").get("deployPolicy", String.class));
  }

  @Test
  public void defaultsToDepthZeroAndTheStrictPolicy() {
    Configuration configuration = underTest.convert(request(null, null));

    assertEquals(Integer.valueOf(0), configuration.attributes("yum").get("repodataDepth", Integer.class));
    assertEquals("STRICT", configuration.attributes("yum").get("deployPolicy", String.class));
    assertNull(configuration.getAttributes().get("yumSigning"));
  }

  @Test
  public void keepsTheStoredPassphraseWhenTheKeyIsUnchanged() {
    ConfigurationData existingConfiguration = new ConfigurationData();
    existingConfiguration.setAttributes(new HashMap<>(Map.of("yumSigning",
        new HashMap<>(Map.of("keypair", "KEY", "passphrase", "secret")))));
    Repository existing = mock(Repository.class);
    when(existing.getConfiguration()).thenReturn(existingConfiguration);
    when(repositoryManager.get("yum-hosted")).thenReturn(existing);

    Configuration unchanged = underTest.convert(request(null, new YumSigningAttributes("KEY", null)));
    Configuration replaced = underTest.convert(request(null, new YumSigningAttributes("OTHER", null)));

    assertEquals("secret", unchanged.attributes("yumSigning").get("passphrase", String.class));
    assertEquals("OTHER", replaced.attributes("yumSigning").get("keypair", String.class));
    assertNull(replaced.attributes("yumSigning").get("passphrase", String.class));
  }

  private static YumHostedRepositoryApiRequest request(final YumAttributes yum, final YumSigningAttributes signing) {
    return new YumHostedRepositoryApiRequest("yum-hosted", true,
        new HostedStorageAttributes("default", true, "ALLOW_ONCE"), null, null, yum, signing);
  }
}
