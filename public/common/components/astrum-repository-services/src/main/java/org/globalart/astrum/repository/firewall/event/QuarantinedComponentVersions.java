
package org.globalart.astrum.repository.firewall.event;

import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.vulnerability.AuditRepositoryComponent;

/**
 * @since 3.35
 */
public class QuarantinedComponentVersions
{
  private final Map<AuditRepositoryComponent, List<String>> auditRepositoryComponents = new HashMap<>();

  public void addComponentVersions(final QuarantinedComponentVersions componentVersions) {
    auditRepositoryComponents.putAll(componentVersions.getAuditRepositoryComponents());
  }

  /**
   * Add vulnerability to the component.
   *
   * @param auditRepositoryComponent {@link AuditRepositoryComponent} quarantined component.
   * @param version of the quarantined component
   */
  public void addRepositoryVersion(
      final AuditRepositoryComponent auditRepositoryComponent,
      @Nullable final String version)
  {
    List<String> versions = auditRepositoryComponents.computeIfAbsent(auditRepositoryComponent, k -> new ArrayList<>());
    if (version != null) {
      versions.add(version);
    }
  }

  public Map<AuditRepositoryComponent, List<String>> getAuditRepositoryComponents() {
    return auditRepositoryComponents;
  }
}
