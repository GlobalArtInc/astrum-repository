
package org.globalart.astrum.internal.capability.storage.datastore.cleanup;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Map.Entry;
import java.util.stream.Collectors;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.capability.CapabilityIdentity;
import org.globalart.astrum.internal.capability.storage.CapabilityStorageItem;
import org.globalart.astrum.internal.capability.storage.upgrade.UpgradeCapabilityStorage;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Remove all capability duplicate records from storage.
 */
@Component
public class CleanupCapabilityDuplicatesService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final UpgradeCapabilityStorage capabilityStorage;

  @Autowired
  public CleanupCapabilityDuplicatesService(final UpgradeCapabilityStorage capabilityStorage) {
    this.capabilityStorage = checkNotNull(capabilityStorage);
  }

  public void doCleanup() {
    Map<CapabilityStorageItem, List<CapabilityIdentity>> duplicateCapabilities = browseCapabilityDuplicates();
    if (duplicateCapabilities.isEmpty()) {
      log.debug("No capabilities duplicates found.");
      return;
    }

    List<CapabilityIdentity> toRemove = new ArrayList<>();
    duplicateCapabilities.forEach((item, duplicates) -> {
      log.info("Cleaning up {} duplicates for {} capability", duplicates.size() - 1, item.getType());

      duplicates.stream()
          .skip(1) // leave one capability in the storage
          .forEach(toRemove::add);
    });

    // Remove all duplicates in a single batched transaction rather than one connection acquisition + commit
    // (fsync on PostgreSQL) per row, which on instances with many duplicates is the startup-outage failure
    // mode ASTRUM-53442 targets.
    int removed = capabilityStorage.removeAll(toRemove);
    log.debug("Removed {} capability duplicate(s)", removed);
  }

  /**
   * Find capability duplicates.
   *
   * @return duplicates capability identities grouped by capability
   */
  @VisibleForTesting
  Map<CapabilityStorageItem, List<CapabilityIdentity>> browseCapabilityDuplicates() {
    return capabilityStorage.getAll()
        .entrySet()
        .stream()
        .collect(Collectors.groupingBy(Entry::getValue))
        .entrySet()
        .stream()
        .filter(f -> f.getValue().size() > 1)
        .collect(Collectors.toMap(
            Entry::getKey,
            entry -> entry.getValue()
                .stream()
                .map(Entry::getKey)
                .collect(Collectors.toList())));
  }
}
