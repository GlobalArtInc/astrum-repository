
package org.globalart.astrum.repository.firewall.event;

import java.util.Set;
import java.util.concurrent.CompletableFuture;

import org.globalart.astrum.repository.vulnerability.AuditRepositoryComponent;

/**
 * @since 3.35
 */
public class QuarantinedComponentVersionsRequest
{
  private final String repositoryName;

  private final Set<AuditRepositoryComponent> repositoryComponents;

  private final CompletableFuture<QuarantinedComponentVersions> quarantinedResult = new CompletableFuture<>();

  public QuarantinedComponentVersionsRequest(
      final String repositoryName,
      final Set<AuditRepositoryComponent> repositoryComponents)
  {
    this.repositoryName = repositoryName;
    this.repositoryComponents = repositoryComponents;
  }

  public String getRepositoryName() {
    return repositoryName;
  }

  public Set<AuditRepositoryComponent> getRepositoryComponents() {
    return repositoryComponents;
  }

  public CompletableFuture<QuarantinedComponentVersions> getQuarantinedResult() {
    return quarantinedResult;
  }
}
