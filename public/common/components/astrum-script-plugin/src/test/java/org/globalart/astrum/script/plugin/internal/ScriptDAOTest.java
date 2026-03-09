
package org.globalart.astrum.script.plugin.internal;

import org.junit.Before;
import org.junit.After;
import org.junit.Rule;
import org.junit.Test;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.script.Script;
import org.globalart.astrum.testdb.DataSessionRule;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.*;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class ScriptDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(ScriptDAO.class);

  private DataSession<?> session;

  private ScriptDAO dao;

  @Before
  public void setUp() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = session.access(ScriptDAO.class);
  }

  @After
  public void tearDown() {
    session.close();
  }

  @Test
  public void testCreateReadUpdateDelete() {
    ScriptData script = new ScriptData();
    script.setName("hello");
    script.setContent("log.info('hello')");

    dao.create(script);

    Script read = dao.read(script.getName()).orElse(null);

    assertThat(read, is(notNullValue()));
    assertThat(read.getName(), is(script.getName()));
    assertThat(read.getType(), is(script.getType()));
    assertThat(read.getContent(), is(script.getContent()));

    script.setContent("log.info('world')");
    dao.update(script);

    Script update = dao.read(script.getName()).orElse(null);

    assertThat(update, is(notNullValue()));
    assertThat(update.getName(), is(script.getName()));
    assertThat(update.getType(), is(script.getType()));
    assertThat(update.getContent(), is(script.getContent()));

    dao.delete(script.getName());

    assertThat(dao.read(script.getName()).isPresent(), is(false));
  }
}
