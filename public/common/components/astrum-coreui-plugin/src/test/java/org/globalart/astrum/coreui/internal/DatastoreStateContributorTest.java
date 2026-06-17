
package org.globalart.astrum.coreui.internal;

import org.globalart.astrum.common.db.DatabaseCheck;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

public class DatastoreStateContributorTest
{
  @Test
  public void datastoreStateContributorExposesIsPostgresqlState() {
    DatabaseCheck dbCheck = mock(DatabaseCheck.class);
    when(dbCheck.isPostgresql()).thenReturn(true);

    DatastoreStateContributor contributor = new DatastoreStateContributor(dbCheck);

    assertThat(contributor.getState().get("datastore.isPostgresql"), is(true));
  }
}
