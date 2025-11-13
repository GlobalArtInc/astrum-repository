
package org.globalart.astrum.internal.security.model;

import java.io.File;
import java.io.IOException;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.security.config.CRole;
import org.globalart.astrum.security.config.SecurityConfiguration;
import org.globalart.astrum.supportzip.ExportSecurityData;
import org.globalart.astrum.supportzip.ImportData;
import org.globalart.astrum.supportzip.datastore.JsonExporter;
import org.springframework.stereotype.Component;
import org.springframework.beans.factory.annotation.Qualifier;

/**
 * Write/Read {@link CRole} data to/from a JSON file.
 *
 * @since 3.29
 */
@Component
@Qualifier("roleExport")
public class RoleExport
    extends JsonExporter
    implements ExportSecurityData, ImportData
{
  private final SecurityConfiguration configuration;

  @Autowired
  public RoleExport(final SecurityConfiguration configuration) {
    this.configuration = configuration;
  }

  @Override
  public void export(final File file) throws IOException {
    log.debug("Export CRole data to {}", file);
    List<CRole> cleanupPolicies = configuration.getRoles();
    exportToJson(cleanupPolicies, file);
  }

  @Override
  public void restore(final File file) throws IOException {
    log.debug("Restoring CRole data from {}", file);
    importFromJson(file, CRoleData.class).forEach(configuration::addRole);
  }
}
