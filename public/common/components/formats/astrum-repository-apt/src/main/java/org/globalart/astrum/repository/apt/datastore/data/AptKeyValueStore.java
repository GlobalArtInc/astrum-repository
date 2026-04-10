
package org.globalart.astrum.repository.apt.datastore.data;

import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.content.kv.KeyValueStore;

import org.springframework.beans.factory.annotation.Autowired;

public class AptKeyValueStore
    extends KeyValueStore<AptKeyValueDAO>
{
  @Autowired
  public AptKeyValueStore(final DataSessionSupplier sessionSupplier, final String contentStoreName) {
    super(sessionSupplier, contentStoreName, AptKeyValueDAO.class);
  }
}
