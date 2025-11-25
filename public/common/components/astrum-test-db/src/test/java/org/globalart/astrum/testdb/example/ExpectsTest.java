
package org.globalart.astrum.testdb.example;

import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.datastore.api.Expects;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Rule;
import org.junit.Test;

import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

/**
 * Test the {@link Expects} annotation, used to declare access types to be registered first.
 */
public class ExpectsTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(TestExpectsDAO.class);

  @SuppressWarnings("java:S2699") // sonar expects assertions, but best to let this exception bubble up
  @Test
  public void testExpectedAccessTypesRegisteredFirst() {
    try (DataSession<?> session = sessionRule.openSession(DEFAULT_DATASTORE_NAME)) {
      // will fail if @Expects is not respected
      session.access(TestItemDAO.class);
      session.getTransaction().commit();
    }
  }
}
