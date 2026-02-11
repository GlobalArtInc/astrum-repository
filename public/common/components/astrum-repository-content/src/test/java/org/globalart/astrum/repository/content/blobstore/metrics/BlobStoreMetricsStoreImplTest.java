
package org.globalart.astrum.repository.content.blobstore.metrics;

import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class BlobStoreMetricsStoreImplTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule(DEFAULT_DATASTORE_NAME).access(BlobStoreMetricsDAO.class);

  @Test
  public void renameMetricsDelegatesToDAO() {
    BlobStoreMetricsStoreImpl store = new BlobStoreMetricsStoreImpl((DataSessionSupplier) sessionRule);
    store.initializeMetrics("old-store");

    store.rename("old-store", "new-store");

    assertThat(store.get("new-store"), is(notNullValue()));
    assertThat(store.get("new-store").getBlobStoreName(), is("new-store"));
    assertThat(store.get("old-store"), is(nullValue()));
  }
}
