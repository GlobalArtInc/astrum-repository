
package org.globalart.astrum.repository.config.internal;

import java.util.Collection;

import org.globalart.astrum.datastore.api.DataSessionSupplier;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationDAO;
import org.globalart.astrum.testdb.DataSessionConfiguration;

import org.hamcrest.core.Is;
import org.junit.jupiter.api.BeforeEach;
import org.globalart.astrum.testdb.DatabaseTest;

import static java.util.Collections.emptySet;
import static org.hamcrest.MatcherAssert.assertThat;

class ConfigurationStoreImplTest
{
  @DataSessionConfiguration(daos = ConfigurationDAO.class)
  private DataSessionSupplier sessionSupplier;

  private ConfigurationStoreImpl underTest;

  @BeforeEach
  void setup() {
    underTest = new ConfigurationStoreImpl(sessionSupplier);
  }

  @DatabaseTest
  void readByNamesShouldBeEmptyWhenRepositoriesIsEmpty() {
    Collection<Configuration> configurations = underTest.readByNames(emptySet());

    assertThat(configurations.isEmpty(), Is.is(true));
  }
}
