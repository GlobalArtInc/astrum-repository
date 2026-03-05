
package org.globalart.astrum.cleanup.storage;

import java.io.File;
import java.io.IOException;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.cleanup.internal.storage.CleanupPolicyData;
import org.globalart.astrum.supportzip.ExportConfigData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link CleanupPolicy} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("cleanupPolicyExport")
public class CleanupPolicyExport
    extends JsonExporter
    implements ExportConfigData, ImportData
{
  private final CleanupPolicyStorage policyStorage;

  @Autowired
  public CleanupPolicyExport(final CleanupPolicyStorage policyStorage) {
    this.policyStorage = policyStorage;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export CleanupPolicy data to {}", file);
    List<CleanupPolicy> cleanupPolicies = policyStorage.getAll();
    exportToJson(cleanupPolicies, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring CleanupPolicy data from {}", file);
    importFromJson(file, CleanupPolicyData.class).forEach(policyStorage::add);
  }
}
