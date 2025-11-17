
package org.globalart.astrum.internal.node.datastore;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.DuplicateKeyException;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.core.IsNot.not;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class NodeIdDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(NodeIdDAO.class);

  private DataSession session;

  private NodeIdDAO dao;

  @Before
  public void setup() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = (NodeIdDAO) session.access(NodeIdDAO.class);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void testSetAndGet() {
    String nodeId = "";
    dao.set(nodeId);
    String readData = dao.get().orElse(null);

    assertThat(readData, not(nullValue()));
    assertThat(readData, is(nodeId));

    nodeId = "";
    readData = dao.get().orElse(null);

    assertThat(readData, not(nullValue()));
    assertThat(readData, is(nodeId));

    dao.clear();
    readData = dao.get().orElse(null);

    assertThat(readData, nullValue());
  }

  @Test
  public void testCreate() {
    String nodeId = "";
    dao.create(nodeId);

    String readData = dao.get().orElse(null);
    assertThat(readData, is(nodeId));
  }

  @Test(expected = DuplicateKeyException.class)
  public void testCreate_fail() {
    String nodeId = "";
    // setup
    dao.create(nodeId);

    // sanity check
    assertThat(dao.get().isPresent(), is(true));

    // call create when it already exists
    dao.create("b");
  }
}
