
package org.globalart.astrum.repository.internal.blobstore;

import java.io.File;
import java.io.IOException;
import java.util.Arrays;
import java.util.Collections;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;
import java.util.stream.Collectors;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.common.entity.EntityUUID;
import org.globalart.astrum.repository.blobstore.BlobStoreConfigurationStore;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.Matchers.allOf;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests validity of Serialization/Deserialization {@link BlobStoreConfiguration}
 * by {@link BlobStoreConfigurationExport}
 */
public class BlobStoreConfigurationExportTest
{
  private final JsonExporter jsonExporter = new JsonExporter();

  private File jsonFile;

  @Before
  public void setup() throws IOException {
    jsonFile = File.createTempFile("BlobStoreConfiguration", ".json");
  }

  @After
  public void tearDown() {
    jsonFile.delete();
  }

  @Test
  public void testExportImportToJson() throws Exception {
    List<BlobStoreConfiguration> configurationData = Arrays.asList(
        generateConfigData("test1", "TEST_1"),
        generateConfigData("test2", "TEST_2"));

    BlobStoreConfigurationStore configurationStore = mock(BlobStoreConfigurationStore.class);
    when(configurationStore.list()).thenReturn(configurationData);

    BlobStoreConfigurationExport exporter = new BlobStoreConfigurationExport(configurationStore);
    exporter.export(jsonFile);
    List<BlobStoreConfigurationData> importedData =
        jsonExporter.importFromJson(jsonFile, BlobStoreConfigurationData.class);

    assertThat(importedData.stream().map(BlobStoreConfiguration::getName).collect(Collectors.toList()),
        containsInAnyOrder("test1", "test2"));
    assertThat(importedData.stream().map(BlobStoreConfiguration::getType).collect(Collectors.toList()),
        containsInAnyOrder("TEST_1", "TEST_2"));
    assertThat(importedData.stream().map(BlobStoreConfiguration::isWritable).collect(Collectors.toList()),
        not(contains(false)));

    List<Map<String, Map<String, Object>>> serializedAttrs = importedData.stream()
        .map(BlobStoreConfiguration::getAttributes)
        .collect(Collectors.toList());
    assertThat(serializedAttrs.toString(), allOf(
        containsString("metadata"),
        containsString("size"),
        containsString("10")));
    // make sure sensitive data is not serialized
    assertThat(serializedAttrs.toString(), not(containsString("admin123")));
  }

  private BlobStoreConfiguration generateConfigData(final String name, final String type) {
    BlobStoreConfigurationData configuration = new BlobStoreConfigurationData();
    configuration.setId(new EntityUUID(UUID.randomUUID()));
    configuration.setName(name);
    configuration.setType(type);
    configuration.setWritable(true);
    Map<String, Map<String, Object>> attributes = new HashMap<>();
    attributes.put("login", Collections.singletonMap("password", "admin123"));
    attributes.put("user", Collections.singletonMap("secret", "admin123"));
    attributes.put("metadata", Collections.singletonMap("size", 10));
    configuration.setAttributes(attributes);

    return configuration;
  }
}
