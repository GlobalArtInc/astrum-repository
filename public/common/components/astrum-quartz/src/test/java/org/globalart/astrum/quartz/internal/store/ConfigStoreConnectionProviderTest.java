
package org.globalart.astrum.quartz.internal.store;

import java.sql.Connection;
import java.sql.SQLException;

import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.startsWith;

/**
 * Test the {@link ConfigStoreConnectionProvider}.
 */
public class ConfigStoreConnectionProviderTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule();

  private ConfigStoreConnectionProvider underTest;

  @Before
  public void setUp() {
    underTest = new ConfigStoreConnectionProvider(sessionRule);
    underTest.initialize();
  }

  @After
  public void tearDown() {
    underTest.shutdown();
    underTest = null;
  }

  @Test
  public void testCanOpenConnection() throws SQLException {
    try (Connection connection = underTest.getConnection()) {
      if (System.getProperty("test.postgres") != null) {
        assertThat(connection.getMetaData().getURL(), startsWith("jdbc:postgresql:"));
      }
      else {
        assertThat(connection.getMetaData().getURL(), is("jdbc:h2:mem:astrum"));
      }
    }
  }
}
