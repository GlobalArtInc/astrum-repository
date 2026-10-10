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

/**
 * Sessions no longer expire by default. A UI settings capability that still carries the former default of
 * 30 minutes is moved to the new default of 0 (never expire); any other value was chosen and stays.
 */
@Component
public class SettingsCapabilitySessionTimeoutMigrationStep_2_166
    implements DatabaseMigrationStep
{
  static final String SESSION_TIMEOUT = "sessionTimeout";

  static final String FORMER_DEFAULT = "30";

  static final String NEVER_EXPIRE = "0";

  private final UpgradeCapabilityStorage capabilityStorage;

  @Autowired
  public SettingsCapabilitySessionTimeoutMigrationStep_2_166(final UpgradeCapabilityStorage capabilityStorage) {
    this.capabilityStorage = checkNotNull(capabilityStorage);
  }

  @Override
  public Optional<String> version() {
    return Optional.of("2.166");
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
          if (FORMER_DEFAULT.equals(properties.get(SESSION_TIMEOUT))) {
            properties.put(SESSION_TIMEOUT, NEVER_EXPIRE);
            capabilityStorage.update(entry.getKey(), capabilityStorageItem);
          }
        });
  }
}
