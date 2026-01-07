
package org.globalart.astrum.quartz.internal.store;

import org.globalart.astrum.quartz.internal.AbstractJobStoreTest;
import org.globalart.astrum.quartz.internal.JobStoreJdbcProvider;
import org.globalart.astrum.quartz.internal.datastore.QuartzDAO;
import org.globalart.astrum.quartz.internal.datastore.QuartzJobDataTypeHandler;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Rule;
import org.quartz.spi.JobStore;

public class JobStoreJdbcStoreProviderTest
    extends AbstractJobStoreTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule()
      .handle(new QuartzJobDataTypeHandler())
      .access(QuartzDAO.class);

  private JobStore jobStore;

  @Override
  protected JobStore createJobStore(final String name) {
    try {
      jobStore = new JobStoreJdbcProvider(new ConfigStoreConnectionProvider(sessionRule), new SimpleNodeAccess(), false)
          .getObject();
    }
    catch (Exception e) {
      throw new RuntimeException(e);
    }
    jobStore.setInstanceId("SINGLE_NODE_TEST");
    jobStore.setInstanceName(name);

    return jobStore;
  }

  @Override
  protected void destroyJobStore(final String name) {
    jobStore.shutdown();
    jobStore = null;
  }
}
