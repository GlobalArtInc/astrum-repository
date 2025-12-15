
package org.globalart.astrum.internal.capability;

import java.io.File;
import java.io.IOException;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.UUID;

import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.common.entity.EntityUUID;
import org.globalart.astrum.internal.capability.storage.CapabilityStorage;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItem;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;

import com.google.common.collect.ImmutableMap;
import org.junit.After;
import org.junit.Before;
import org.junit.Test;

import static java.util.stream.Collectors.toList;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

/**
 * Tests validity of Serialization/Deserialization CapabilityStorage by {@link CapabilityStorageExport}
 */
public class CapabilityStorageExportTest
{
  private final JsonExporter jsonExporter = new JsonExporter();

  private File binFile;

  @Before
  public void setup() throws IOException {
    binFile = File.createTempFile("CapabilityStorage", ".json");
  }

  @After
  public void tearDown() {
    binFile.delete();
  }

  @Test
  public void testExportImportToJson() throws Exception {
    Map<CapabilityIdentity, CapabilityStorageItem> capabilities = new HashMap<>(2);
    capabilities.put(new CapabilityIdentity("one"), createCapabilityWithSensitiveData());
    capabilities.put(new CapabilityIdentity("two"), createCapabilityWithoutSensitiveData());

    CapabilityStorage capabilityStorage = mock(CapabilityStorage.class);
    when(capabilityStorage.getAll()).thenReturn(capabilities);

    CapabilityStorageExport exporter = new CapabilityStorageExport(capabilityStorage);
    exporter.export(binFile);
    List<CapabilityStorageItemData> importedItems =
        jsonExporter.importFromJson(binFile, CapabilityStorageItemData.class);

    assertThat(importedItems.stream().map(CapabilityStorageItem::getVersion).collect(toList()),
        containsInAnyOrder(1, 2));
    assertThat(importedItems.stream().map(CapabilityStorageItem::getType).collect(toList()),
        containsInAnyOrder("testing 1", "testing 2"));
    assertThat(importedItems.stream().map(CapabilityStorageItem::isEnabled).collect(toList()),
        containsInAnyOrder(true, false));
    assertThat(importedItems.stream().map(CapabilityStorageItem::getNotes).collect(toList()),
        containsInAnyOrder("notes 1", "notes 2"));
    List<Map<String, String>> attributes =
        importedItems.stream().map(CapabilityStorageItem::getProperties).collect(toList());
    assertThat(attributes.toString(), containsString("Capability"));
    assertThat(attributes.toString(), containsString("Testing"));
    // make sure sensitive data is not serialized
    assertThat(attributes.toString(), not(containsString("admin123")));
  }

  private CapabilityStorageItemData createCapabilityWithSensitiveData() {
    CapabilityStorageItemData item = new CapabilityStorageItemData();
    Map<String, String> sensitiveData = ImmutableMap.of(
        "password", "admin123",
        "secret", "admin123",
        "bearerTokenId", "admin123");
    item.setId(new EntityUUID(UUID.randomUUID()));
    item.setVersion(2);
    item.setType("testing 2");
    item.setEnabled(false);
    item.setNotes("notes 2");
    item.setProperties(sensitiveData);

    return item;
  }

  private CapabilityStorageItemData createCapabilityWithoutSensitiveData() {
    CapabilityStorageItemData item = new CapabilityStorageItemData();
    Map<String, String> sensitiveData = ImmutableMap.of(
        "title", "Capability",
        "name", "Testing");
    item.setId(new EntityUUID(UUID.randomUUID()));
    item.setVersion(1);
    item.setType("testing 1");
    item.setEnabled(true);
    item.setNotes("notes 1");
    item.setProperties(sensitiveData);

    return item;
  }
}
