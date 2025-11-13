
package org.globalart.astrum.internal.security.model;

import java.io.File;
import java.io.IOException;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.security.config.CPrivilege;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.supportzip.ExportSecurityData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link CPrivilege} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("privilegeExport")
public class PrivilegeExport
    extends JsonExporter
    implements ExportSecurityData, ImportData
{
  private final SecurityConfiguration configuration;

  @Autowired
  public PrivilegeExport(final SecurityConfiguration configuration) {
    this.configuration = configuration;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export CPrivilege data to {}", file);
    List<CPrivilege> cleanupPolicies = configuration.getPrivileges();
    exportToJson(cleanupPolicies, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring CPrivilege data from {}", file);
    importFromJson(file, CPrivilegeData.class).forEach(configuration::addPrivilege);
  }
}
