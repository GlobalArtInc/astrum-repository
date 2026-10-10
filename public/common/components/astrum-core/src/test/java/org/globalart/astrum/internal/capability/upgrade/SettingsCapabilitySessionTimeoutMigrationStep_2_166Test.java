package org.globalart.astrum.internal.capability.upgrade;

import java.util.HashMap;
import java.util.Map;

import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItem;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItemData;
import org.globalart.astrum.internal.capability.storage.upgrade.UpgradeCapabilityStorage;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.ArgumentCaptor;
import org.mockito.Captor;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import static java.util.Collections.singletonMap;
import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@ExtendWith(MockitoExtension.class)
class SettingsCapabilitySessionTimeoutMigrationStep_2_166Test
{
  @Mock
  private UpgradeCapabilityStorage capabilityStorage;

  @Captor
  private ArgumentCaptor<CapabilityStorageItem> itemCaptor;

  private SettingsCapabilitySessionTimeoutMigrationStep_2_166 underTest;

  @BeforeEach
  void setUp() {
    underTest = new SettingsCapabilitySessionTimeoutMigrationStep_2_166(capabilityStorage);
  }

  @Test
  void migrate_formerDefaultBecomesNeverExpire() throws Exception {
    CapabilityIdentity id = new CapabilityIdentity("id-1");
    Map<String, String> properties = new HashMap<>();
    properties.put("sessionTimeout", "30");
    properties.put("requestTimeout", "60");
    when(capabilityStorage.getAll()).thenReturn(singletonMap(id, item("rapture.settings", properties)));

    underTest.migrate(null);

    verify(capabilityStorage).update(eq(id), itemCaptor.capture());
    assertThat(itemCaptor.getValue().getProperties())
        .containsEntry("sessionTimeout", "0")
        .containsEntry("requestTimeout", "60");
  }

  @Test
  void migrate_keepsAChosenTimeout() throws Exception {
    Map<String, String> properties = new HashMap<>();
    properties.put("sessionTimeout", "45");
    when(capabilityStorage.getAll())
        .thenReturn(singletonMap(new CapabilityIdentity("id-1"), item("rapture.settings", properties)));

    underTest.migrate(null);

    verify(capabilityStorage, never()).update(any(), any());
  }

  @Test
  void migrate_noRaptureSettings_doesNothing() throws Exception {
    when(capabilityStorage.getAll())
        .thenReturn(singletonMap(new CapabilityIdentity("id-2"), item("other.capability", new HashMap<>())));

    underTest.migrate(null);

    verify(capabilityStorage, never()).update(any(), any());
  }

  @Test
  void version() {
    assertThat(underTest.version()).contains("2.166");
  }

  private static CapabilityStorageItem item(final String type, final Map<String, String> properties) {
    CapabilityStorageItemData data = new CapabilityStorageItemData();
    data.setVersion(1);
    data.setType(type);
    data.setEnabled(true);
    data.setNotes("notes");
    data.setProperties(properties);
    return data;
  }
}
