
package org.globalart.astrum.internal.jwt.datastore;

import java.util.Optional;
import java.util.function.Consumer;
import java.util.function.Function;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.SingletonDataAccess;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class JwtSecretDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(JwtSecretDAO.class);

  private DataSession<?> session;

  private JwtSecretDAO dao;

  @Before
  public void setup() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = session.access(JwtSecretDAO.class);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void testSetAndGet() {
    Optional<String> emptySecret = withDao(SingletonDataAccess::get);
    assertThat(emptySecret.isPresent(), is(false));

    callDao(dao -> dao.setIfEmpty("secret"));
    Optional<String> initialSecret = withDao(SingletonDataAccess::get);

    assertThat(initialSecret.isPresent(), is(true));
    assertThat(initialSecret.get(), is("secret"));

    callDao(dao -> dao.set("new-secret"));
    Optional<String> newSecret = withDao(SingletonDataAccess::get);

    assertThat(newSecret.isPresent(), is(true));
    assertThat(newSecret.get(), is("new-secret"));
  }

  private void callDao(final Consumer<JwtSecretDAO> fn) {
    fn.accept(dao);
    session.getTransaction().commit();
  }

  private <T> T withDao(final Function<JwtSecretDAO, T> fn) {
    T result = fn.apply(dao);
    session.getTransaction().commit();
    return result;
  }
}
