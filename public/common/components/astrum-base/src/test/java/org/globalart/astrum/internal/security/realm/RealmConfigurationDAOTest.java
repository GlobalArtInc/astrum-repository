
package org.globalart.astrum.internal.security.realm;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static java.util.Arrays.asList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.core.IsNot.not;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class RealmConfigurationDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(RealmConfigurationDAO.class);

  private DataSession session;

  private RealmConfigurationDAO dao;

  @Before
  public void setup() throws Exception {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = (RealmConfigurationDAO) session.access(RealmConfigurationDAO.class);
  }

  @After
  public void cleaup() {
    session.close();
  }

  @Test
  public void testSetAndGet() {
    RealmConfigurationData config = new RealmConfigurationData();
    config.setRealmNames(asList("hello", "world"));
    dao.set(config);
    RealmConfigurationData readData = dao.get().orElse(null);

    assertThat(readData, not(nullValue()));
    assertThat(readData.getRealmNames(), containsInAnyOrder("hello", "world"));

    config.setRealmNames(asList("foo", "bar"));
    dao.set(config);
    readData = dao.get().orElse(null);
    assertThat(readData, not(nullValue()));
    assertThat(readData.getRealmNames(), containsInAnyOrder("foo", "bar"));
  }
}
