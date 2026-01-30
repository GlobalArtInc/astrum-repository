
package org.globalart.astrum.repository.internal.blobstore;

import java.util.Collection;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.stream.IntStream;

import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.group.BlobStoreGroup;
import org.globalart.astrum.datastore.api.DataSession;
import org.globalart.astrum.testdb.DataSessionRule;

import org.junit.Before;
import org.junit.Rule;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.globalart.astrum.datastore.api.DataStoreManager.DEFAULT_DATASTORE_NAME;

public class BlobStoreConfigurationDAOTest
{
  @Rule
  public DataSessionRule sessionRule = new DataSessionRule().access(BlobStoreConfigurationDAO.class);

  private DataSession<?> session;

  BlobStoreConfigurationDAO mapper;

  @Before
  public void setup() {
    session = sessionRule.openSession(DEFAULT_DATASTORE_NAME);
    mapper = session.access(BlobStoreConfigurationDAO.class);
  }

  @Test
  public void testBrowsing() {
    IntStream.range(1, 6).forEach(it -> mapper.create(blobStore("name-" + it, "type-" + it, new HashMap<>())));

    Collection<BlobStoreConfigurationData> items = collect(mapper.browse());
    assertThat(items, hasSize(5));
  }

  @Test
  public void testCRUD() {
    BlobStoreConfigurationData config = blobStore("name", "type", new HashMap<>());
    mapper.create(config);

    BlobStoreConfigurationData readBack = mapper.readByName(config.getName()).orElse(null);
    assertThat(readBack, notNullValue());
    assertThat(readBack.getName(), is(config.getName()));
    assertThat(readBack.getType(), is(config.getType()));

    // update
    config.setType("newType");
    mapper.update(config);

    // read back
    BlobStoreConfigurationData updated = mapper.readByName(config.getName()).orElse(null);
    assertThat(updated, notNullValue());
    assertThat(updated.getName(), is(config.getName()));
    assertThat(updated.getType(), is("newType"));

    // delete it
    assertTrue(mapper.deleteByName(config.getName()));
    assertFalse(mapper.readByName(config.getName()).isPresent());
    assertThat(collect(mapper.browse()), hasSize(0));
  }

  @Test
  public void testFindingParent() {
    List<String> memberNames = List.of("A", "B", "C");

    List<BlobStoreConfigurationData> members = memberNames.stream()
        .map(it -> blobStore(it, "file", new HashMap<>()))
        .toList();
    BlobStoreConfigurationData config =
        blobStore("parent", BlobStoreGroup.TYPE, Map.of("group", Map.of("members", memberNames)));

    members.forEach(mapper::create);
    mapper.create(config);

    Optional<BlobStoreConfiguration> foundConfig = mapper.findCandidateParents("A").stream().findFirst();
    // the correct values and all member relationships exist
    assertThat(foundConfig.get().getName(), is("parent"));

    // a config with no parent is used to find a parent"
    Optional<BlobStoreConfiguration> noParent = mapper.findCandidateParents("42").stream().findFirst();
    assertFalse(noParent.isPresent());
  }

  private static <E> Collection<E> collect(final Iterable<E> iterable) {
    return (Collection<E>) iterable;
  }

  private static BlobStoreConfigurationData blobStore(
      final String name,
      final String type,
      final Map<String, Map<String, Object>> attributes)
  {
    BlobStoreConfigurationData data = new BlobStoreConfigurationData();
    data.setName(name);
    data.setType(type);
    data.setAttributes(attributes);
    return data;
  }
}
