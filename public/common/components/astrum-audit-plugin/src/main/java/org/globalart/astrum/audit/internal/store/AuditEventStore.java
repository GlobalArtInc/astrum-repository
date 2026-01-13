
package org.globalart.astrum.audit.internal.store;

import java.time.OffsetDateTime;
import java.util.Collection;
import java.util.List;

import org.springframework.beans.factory.annotation.Autowired;
import org.globalart.astrum.datastore.ConfigStoreSupport;
import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.transaction.Transactional;

import org.springframework.stereotype.Component;

/**
 * Store for persisted audit events, backed by MyBatis.
 */
@Component
public class AuditEventStore
    extends ConfigStoreSupport<AuditEventDAO>
{
  @Autowired
  public AuditEventStore(final DataSessionSupplier sessionSupplier) {
    super(sessionSupplier);
  }

  @Transactional
  public void insert(final AuditEventData data) {
    dao().insert(data);
  }

  @Transactional
  public List<AuditEventData> findAll(
      final String domain,
      final String type,
      final String initiator,
      final String repositoryName,
      final OffsetDateTime startDate,
      final OffsetDateTime endDate,
      final int limit,
      final int offset)
  {
    return dao().findAll(domain, type, initiator, repositoryName, startDate, endDate, limit, offset);
  }

  @Transactional
  public int count(
      final String domain,
      final String type,
      final String initiator,
      final String repositoryName,
      final OffsetDateTime startDate,
      final OffsetDateTime endDate)
  {
    return dao().count(domain, type, initiator, repositoryName, startDate, endDate);
  }

  @Transactional
  public List<AuditEventData> findByDomains(
      final Collection<String> domains,
      final String type,
      final String initiator,
      final String repositoryName,
      final OffsetDateTime startDate,
      final OffsetDateTime endDate,
      final int limit,
      final int offset)
  {
    return dao().findByDomains(domains, type, initiator, repositoryName, startDate, endDate, limit, offset);
  }

  @Transactional
  public int countByDomains(
      final Collection<String> domains,
      final String type,
      final String initiator,
      final String repositoryName,
      final OffsetDateTime startDate,
      final OffsetDateTime endDate)
  {
    return dao().countByDomains(domains, type, initiator, repositoryName, startDate, endDate);
  }

  /**
   * Delete a single batch of rows older than {@code cutoff}. Returns the number of rows deleted.
   * The batch bound protects against long-lived row locks and unbounded transaction size; callers
   * should loop until the returned count is zero to fully prune the backlog.
   */
  @Transactional
  public int deleteOlderThan(final OffsetDateTime cutoff, final int batchSize) {
    return dao().deleteOlderThan(cutoff, batchSize);
  }
}
