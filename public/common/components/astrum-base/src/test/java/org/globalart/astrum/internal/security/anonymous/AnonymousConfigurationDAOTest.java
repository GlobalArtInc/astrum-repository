
package org.globalart.astrum.internal.security.anonymous;

import org.globalart.astrum.datastore.api.DataSession;
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

public class AnonymousConfigurationDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(AnonymousConfigurationDAO.class);

  private DataSession session;

  private AnonymousConfigurationDAO dao;

  @Before
  public void setup() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = (AnonymousConfigurationDAO) session.access(AnonymousConfigurationDAO.class);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void testSetAndGet() {
    AnonymousConfigurationData config = new AnonymousConfigurationData();
    config.setEnabled(true);
    config.setUserId("anon");
    config.setRealmName("local");
    dao.set(config);
    AnonymousConfigurationData readData = dao.get().orElse(null);

    assertThat(readData, not(nullValue()));
    assertThat(readData.isEnabled(), is(true));
    assertThat(readData.getUserId(), is("anon"));
    assertThat(readData.getRealmName(), is("local"));

    config.setEnabled(false);
    config.setUserId("anonymous");
    config.setRealmName("remote");
    dao.set(config);
    readData = dao.get().orElse(null);

    assertThat(readData, not(nullValue()));
    assertThat(readData.isEnabled(), is(false));
    assertThat(readData.getUserId(), is("anonymous"));
    assertThat(readData.getRealmName(), is("remote"));

    dao.clear();
    readData = dao.get().orElse(null);

    assertThat(readData, nullValue());
  }
}
