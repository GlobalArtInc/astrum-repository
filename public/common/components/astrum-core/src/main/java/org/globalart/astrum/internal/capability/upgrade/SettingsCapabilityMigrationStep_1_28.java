
package org.globalart.astrum.internal.capability.upgrade;

import java.sql.Connection;
import java.util.Map;
import java.util.Optional;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItem;
import org.globalart.astrum.internal.capability.storage.upgrade.UpgradeCapabilityStorage;
import org.globalart.astrum.upgrade.datastore.DatabaseMigrationStep;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.stereotype.Component;

@Component
public class SettingsCapabilityMigrationStep_1_28
    implements DatabaseMigrationStep
{
  private final UpgradeCapabilityStorage capabilityStorage;

  @Autowired
  public SettingsCapabilityMigrationStep_1_28(final UpgradeCapabilityStorage capabilityStorage) {
    this.capabilityStorage = checkNotNull(capabilityStorage);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("1.28");
  }

  @Override
  public void migrate(final Connection connection) throws Exception {
    capabilityStorage.getAll()
        .entrySet()
        .stream()
        .filter(e -> e.getValue().getType().equals("rapture.settings"))
        .findFirst()
        .ifPresent(entry -> {
          CapabilityStorageItem capabilityStorageItem = entry.getValue();
          Map<String, String> properties = capabilityStorageItem.getProperties();
          properties.putIfAbsent("requestTimeout", "60");
          properties.putIfAbsent("longRequestTimeout", "180");
          capabilityStorage.update(entry.getKey(), capabilityStorageItem);
        });
  }
}
