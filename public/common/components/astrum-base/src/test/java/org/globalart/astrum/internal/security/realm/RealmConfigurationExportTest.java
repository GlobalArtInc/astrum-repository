
package org.globalart.astrum.internal.security.realm;

import java.io.File;
import java.io.IOException;
import java.util.Optional;

import org.globalart.astrum.security.realm.RealmConfiguration;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import com.google.common.collect.ImmutableList;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests validity of Serialization/Deserialization {@link RealmConfiguration} by {@link RealmConfigurationExport}
 */
public class RealmConfigurationExportTest
{
  private final JsonExporter jsonExporter = new JsonExporter();

  private File jsonFile;

  @Before
  public void setup() throws IOException {
    jsonFile = File.createTempFile("RealmConfiguration", ".json");
  }

  @After
  public void tearDown() {
    jsonFile.delete();
  }

  @Test
  public void testExportImportToJson() throws Exception {
    RealmConfiguration configuration = createRealmConfiguration();

    RealmConfigurationStoreImpl source = mock(RealmConfigurationStoreImpl.class);
    when(source.load()).thenReturn(configuration);

    RealmConfigurationExport exporter = new RealmConfigurationExport(source);
    exporter.export(jsonFile);
    Optional<RealmConfigurationData> importedData =
        jsonExporter.importObjectFromJson(jsonFile, RealmConfigurationData.class);

    assertTrue(importedData.isPresent());
    assertThat(importedData.get().getRealmNames(), containsInAnyOrder("MockRealmA", "MockRealmB"));
  }

  private RealmConfiguration createRealmConfiguration() {
    RealmConfigurationData configuration = new RealmConfigurationData();
    configuration.setRealmNames(ImmutableList.of("MockRealmA", "MockRealmB"));

    return configuration;
  }
}
