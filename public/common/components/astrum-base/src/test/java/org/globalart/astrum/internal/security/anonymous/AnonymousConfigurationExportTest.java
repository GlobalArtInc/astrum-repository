
package org.globalart.astrum.internal.security.anonymous;

import java.io.File;
import java.io.IOException;
import java.util.Optional;

import org.globalart.astrum.security.anonymous.AnonymousConfiguration;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests validity of Serialization/Deserialization AnonymousConfigurationData by {@link AnonymousConfigurationExport}
 */
public class AnonymousConfigurationExportTest
{
  private final JsonExporter jsonExporter = new JsonExporter();

  private File binFile;

  @Before
  public void setup() throws IOException {
    binFile = File.createTempFile("AnonymousConfiguration", ".json");
  }

  @After
  public void tearDown() {
    binFile.delete();
  }

  @Test
  public void testExportImportToJson() throws Exception {
    AnonymousConfiguration configuration = buildAnonymousConfiguration();
    AnonymousConfigurationStore configurationStore = mock(AnonymousConfigurationStore.class);
    when(configurationStore.load()).thenReturn(configuration);

    AnonymousConfigurationExport exporter = new AnonymousConfigurationExport(configurationStore);
    exporter.export(binFile);
    Optional<AnonymousConfigurationData> importedItem =
        jsonExporter.importObjectFromJson(binFile, AnonymousConfigurationData.class);
    assertTrue(importedItem.isPresent());
    assertThat(importedItem.get(), is(configuration));
  }

  private AnonymousConfiguration buildAnonymousConfiguration() {
    AnonymousConfiguration configuration = new AnonymousConfigurationData();
    configuration.setUserId("userId");
    configuration.setRealmName("realName");
    configuration.setEnabled(true);

    return configuration;
  }
}
