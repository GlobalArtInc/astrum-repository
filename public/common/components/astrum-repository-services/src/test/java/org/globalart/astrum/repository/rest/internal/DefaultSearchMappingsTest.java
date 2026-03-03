
package org.globalart.astrum.repository.rest.internal;

import java.util.stream.StreamSupport;

import org.globalart.astrum.repository.rest.SearchMapping;
import org.globalart.astrum.repository.rest.sql.SearchField;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.junit.Assert.assertTrue;

/**
 * Tests for {@link DefaultSearchMappings}.
 */
public class DefaultSearchMappingsTest
{
  private DefaultSearchMappings underTest;

  @Before
  public void setUp() {
    underTest = new DefaultSearchMappings();
  }

  @Test
  public void testGet_ReturnsSearchMappings() {
    Iterable<SearchMapping> mappings = underTest.get();

    assertThat(mappings, is(notNullValue()));
    assertTrue("Should contain at least one mapping",
        StreamSupport.stream(mappings.spliterator(), false).findAny().isPresent());
  }

  @Test
  public void testGet_ContainsLastUpdatedMapping() {
    Iterable<SearchMapping> mappings = underTest.get();

    boolean hasLastUpdatedMapping = StreamSupport.stream(mappings.spliterator(), false)
        .anyMatch(mapping -> "last_updated".equals(mapping.getAlias())
            && "last_modified".equals(mapping.getAttribute())
            && SearchField.LAST_MODIFIED.equals(mapping.getField()));

    assertTrue("Should contain last_updated mapping to LAST_MODIFIED field", hasLastUpdatedMapping);
  }

  @Test
  public void testGet_ContainsLastBlobUpdatedMapping() {
    Iterable<SearchMapping> mappings = underTest.get();

    boolean hasLastBlobUpdatedMapping = StreamSupport.stream(mappings.spliterator(), false)
        .anyMatch(mapping -> "lastBlobUpdated".equals(mapping.getAlias())
            && "last_modified".equals(mapping.getAttribute())
            && SearchField.LAST_MODIFIED.equals(mapping.getField()));

    assertTrue("Should contain lastBlobUpdated mapping to LAST_MODIFIED field", hasLastBlobUpdatedMapping);
  }

  @Test
  public void testGet_ContainsExpectedCoreMappings() {
    Iterable<SearchMapping> mappings = underTest.get();

    // Verify some core mappings exist
    boolean hasRepositoryMapping = StreamSupport.stream(mappings.spliterator(), false)
        .anyMatch(mapping -> "repository".equals(mapping.getAlias()));
    boolean hasFormatMapping = StreamSupport.stream(mappings.spliterator(), false)
        .anyMatch(mapping -> "format".equals(mapping.getAlias()));
    boolean hasGroupMapping = StreamSupport.stream(mappings.spliterator(), false)
        .anyMatch(mapping -> "group".equals(mapping.getAlias()));

    assertTrue("Should contain repository mapping", hasRepositoryMapping);
    assertTrue("Should contain format mapping", hasFormatMapping);
    assertTrue("Should contain group mapping", hasGroupMapping);
  }

  @Test
  public void testGet_ContainsRepositoryNameAlias_ASTRUM_52827() {
    // The Search results grid in the UI sends sort property "repositoryName"
    // (matching the column's dataIndex). Both "repository" and "repositoryName"
    // must resolve to SearchField.REPOSITORY_NAME so that sorting by the
    // Repository column does not raise "Invalid sort field".
    Iterable<SearchMapping> mappings = underTest.get();

    boolean hasRepositoryNameMapping = StreamSupport.stream(mappings.spliterator(), false)
        .anyMatch(mapping -> "repositoryName".equals(mapping.getAlias())
            && SearchField.REPOSITORY_NAME.equals(mapping.getField()));

    assertTrue("Should contain repositoryName alias mapped to REPOSITORY_NAME field",
        hasRepositoryNameMapping);
  }
}
