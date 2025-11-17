
package org.globalart.astrum.internal.node.datastore;

import java.util.Collections;
import java.util.Optional;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.common.node.NodeAccess;
import org.globalart.astrum.datastore.api.DataStoreManager;
import org.globalart.astrum.testdb.DataSessionRule;
import org.globalart.astrum.transaction.UnitOfWork;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;

/**
 * Tests for local {@link NodeAccess}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class LocalNodeAccessTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(NodeIdDAO.class);

  @Mock
  private EventManager eventManager;

  private NodeAccess nodeAccess;

  private NodeIdStoreImpl store;

  @Before
  public void setUp() throws Exception {
    store = new NodeIdStoreImpl(sessionRule);
    store.setDependencies(eventManager);
    nodeAccess = new LocalNodeAccess(store);

    UnitOfWork.beginBatch(() -> sessionRule.openSession(DataStoreManager.DEFAULT_DATASTORE_NAME));
  }

  @After
  public void tearDown() throws Exception {
    if (nodeAccess != null) {
      nodeAccess.stop();
    }
    UnitOfWork.end();
  }

  @Test
  public void testGeneratesId() throws Exception {
    nodeAccess.start();

    Optional<String> nodeId = store.get();
    assertThat(nodeId.isPresent(), is(true));
  }

  @Test
  public void testUsesDatabaseId() throws Exception {
    store.set("foo");
    nodeAccess.start();

    assertThat(nodeAccess.getId(), is("foo"));
  }

  @Test
  public void localIsOldestNode() throws Exception {
    nodeAccess.start();
    assertThat(nodeAccess.isOldestNode(), is(true));
  }

  @Test
  public void getMemberAliasesKeyValueEqualToIdentity() throws Exception {
    nodeAccess.start();
    assertThat(nodeAccess.getMemberAliases(),
        equalTo(Collections.singletonMap(nodeAccess.getId(), nodeAccess.getId())));
  }
}
