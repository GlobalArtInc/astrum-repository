
package org.globalart.astrum.internal.node;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class DeploymentIdDAOTest
{
  private static final String DEPLOYMENT_ID = "aslfsd";

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(DeploymentIdDAO.class);

  private DataSession session;

  private DeploymentIdDAO dao;

  @Before
  public void setup() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = (DeploymentIdDAO) session.access(DeploymentIdDAO.class);
    dao.set(DEPLOYMENT_ID);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void testGet() {
    String deploymentId = dao.get().orElse(null);

    assertThat(deploymentId, is(DEPLOYMENT_ID));
  }

  @Test
  public void testSet() {
    dao.set("DEPLOYMENT_ID");
    String deploymentId = dao.get().orElse(null);

    assertThat(deploymentId, is("DEPLOYMENT_ID"));
  }
}
