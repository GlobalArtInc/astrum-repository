
package org.globalart.astrum.internal.selector;

import com.google.common.collect.ImmutableMap;
import org.junit.After;
import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.testdb.DataSessionRule;

import java.util.List;
import java.util.Optional;
import java.util.stream.StreamSupport;

import static java.util.stream.Collectors.toList;
import static org.junit.Assert.*;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class SelectorConfigurationDAOTest
{

  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(SelectorConfigurationDAO.class);

  private DataSession<?> session;

  private SelectorConfigurationDAO dao;

  @Before
  public void setup() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    dao = session.access(SelectorConfigurationDAO.class);
  }

  @After
  public void cleanup() {
    session.close();
  }

  @Test
  public void createReadUpdateDelete() {
    // Create a SelectorConfiguration
    SelectorConfigurationData config = new SelectorConfigurationData();
    config.setName("name");
    config.setType("type");
    config.setAttributes(ImmutableMap.of("foo", "bar"));

    // Create the configuration
    dao.create(config);

    // Read the configuration
    Optional<SelectorConfigurationData> readValue = dao.read(config.getName());
    assertTrue(readValue.isPresent());
    SelectorConfiguration read = readValue.get();
    assertEquals(config.getName(), read.getName());
    assertEquals(config.getDescription(), read.getDescription());
    assertEquals(config.getAttributes(), read.getAttributes());

    // Update the configuration
    config.setDescription("description");
    config.getAttributes().put("foo", "baz");
    dao.update(config);

    // Read the updated configuration
    readValue = dao.read(config.getName());
    assertTrue(readValue.isPresent());
    SelectorConfiguration update = readValue.get();
    assertEquals(config.getName(), update.getName());
    assertEquals(config.getDescription(), update.getDescription());
    assertEquals(config.getAttributes(), update.getAttributes());

    // Delete the configuration
    dao.delete(config.getName());

    // Read the deleted configuration
    readValue = dao.read(config.getName());
    assertFalse(readValue.isPresent());
  }

  @Test
  public void browseReturnsCorrectItems() {
    // Create multiple configurations
    for (int i = 1; i <= 5; i++) {
      SelectorConfigurationData config = new SelectorConfigurationData();
      config.setName("name-" + i);
      config.setType("type-" + i);
      config.setDescription("description-" + i);
      config.setAttributes(ImmutableMap.of());
      dao.create(config);
    }

    // Browse the configurations
    List<SelectorConfiguration> configs = StreamSupport.stream(dao.browse().spliterator(), false).collect(toList());
    assertEquals(5, configs.size());
  }
}
