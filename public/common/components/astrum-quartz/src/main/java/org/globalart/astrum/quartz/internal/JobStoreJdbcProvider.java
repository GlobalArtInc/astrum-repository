
package org.globalart.astrum.quartz.internal;

import java.sql.Connection;
import java.sql.SQLException;

import org.globalart.astrum.common.lifecycle.LifecycleSupport;
import org.globalart.astrum.common.app.FeatureFlags;
import org.globalart.astrum.common.app.ManagedLifecycle;
import org.globalart.astrum.common.node.NodeAccess;

import org.springframework.beans.factory.annotation.Autowired;
import org.quartz.impl.jdbcjobstore.HSQLDBDelegate;
import org.quartz.impl.jdbcjobstore.JobStoreTX;
import org.quartz.impl.jdbcjobstore.PostgreSQLDelegate;
import org.quartz.impl.jdbcjobstore.StdJDBCDelegate;
import org.quartz.spi.JobStore;
import org.quartz.utils.ConnectionProvider;
import org.quartz.utils.DBConnectionManager;
import org.springframework.beans.factory.FactoryBean;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.boot.autoconfigure.condition.ConditionalOnProperty;
import org.springframework.context.annotation.Lazy;
import org.springframework.core.Ordered;
import org.springframework.core.annotation.Order;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.common.app.ManagedLifecycle.Phase.SCHEMAS;

/**
 * {@link JobStore} implementation that uses JDBC.
 *
 * @since 3.19
 */
@Lazy
@ConditionalOnProperty(name = "astrum.quartz.jobstore.jdbc", havingValue = "true")
@ManagedLifecycle(phase = SCHEMAS)
@Component
@Order(Ordered.HIGHEST_PRECEDENCE)
public class JobStoreJdbcProvider
    extends LifecycleSupport
    implements FactoryBean<JobStore>
{
  private static final String QUARTZ_DS = "quartzDS";

  private final ConnectionProvider connectionProvider;

  private final boolean datastoreClustered;

  private volatile JobStore jobStore;

  private final NodeAccess nodeAccess;

  @Autowired
  public JobStoreJdbcProvider(
      final ConnectionProvider connectionProvider,
      final NodeAccess nodeAccess,
      @Value(FeatureFlags.DATASTORE_CLUSTERED_ENABLED_NAMED_VALUE) final boolean datastoreClustered)
  {
    this.connectionProvider = connectionProvider;
    this.nodeAccess = nodeAccess;
    this.datastoreClustered = datastoreClustered;
  }

  @Override
  protected void doStop() {
    if (jobStore != null) {
      jobStore.shutdown();
      jobStore = null;
      try {
        connectionProvider.shutdown();
      }
      catch (SQLException e) {
        log.error("Error while stopping job store", e);
      }
    }
  }

  @Lazy
  @Override
  public JobStore getObject() throws Exception {
    JobStore localRef = jobStore;
    if (localRef == null) {
      synchronized (this) {
        localRef = jobStore;
        if (localRef == null) {
          jobStore = localRef = createJobStore();
        }
      }
    }
    return localRef;
  }

  @Override
  public Class<?> getObjectType() {
    return JobStore.class;
  }

  private JobStore createJobStore() {
    try {
      connectionProvider.initialize();
      DBConnectionManager.getInstance().addConnectionProvider(QUARTZ_DS, connectionProvider);
      JobStoreTX delegate = new JobStoreTX();
      delegate.setDataSource(QUARTZ_DS);

      if (datastoreClustered) {
        log.info("Running Quartz in clustered mode");
        delegate.setIsClustered(true);
        delegate.setInstanceName(nodeAccess.getId());
        delegate.setInstanceId("AUTO");
      }

      delegate.setDriverDelegateClass(getDriverDelegateClass());
      return delegate;
    }
    catch (Exception e) {
      log.error("Unable create job store", e);
      return null;
    }
  }

  private String getDatabaseId() throws SQLException {
    try (Connection con = connectionProvider.getConnection()) {
      return con.getMetaData().getDatabaseProductName();
    }
  }

  private String getDriverDelegateClass() throws SQLException {
    switch (getDatabaseId()) {
      case "H2":
        return HSQLDBDelegate.class.getName();
      case "PostgreSQL":
        return PostgreSQLDelegate.class.getName();
      default:
        return StdJDBCDelegate.class.getName();
    }
  }
}
