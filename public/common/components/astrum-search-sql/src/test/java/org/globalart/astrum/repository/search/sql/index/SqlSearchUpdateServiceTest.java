
package org.globalart.astrum.repository.search.sql.index;

import java.util.HashMap;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.search.sql.store.SearchStore;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;

import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.mockito.Mock;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.lenient;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.search.index.SearchUpdateService.SEARCH_INDEX_OUTDATED;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.junit.jupiter.MockitoExtension;

@ExtendWith(MockitoExtension.class)
public class SqlSearchUpdateServiceTest
{
  @Mock
  private Repository repository;

  @Mock
  private ContentFacet contentFacet;

  @Mock
  private SearchStore searchTableStore;

  private NestedAttributesMap attributes = new NestedAttributesMap("root", new HashMap<>());

  private SqlSearchUpdateService underTest;

  @BeforeEach
  public void setup() {
    when(repository.getType()).thenReturn(new HostedType());
    lenient().when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    lenient().when(contentFacet.attributes()).thenReturn(attributes);

    underTest = new SqlSearchUpdateService(searchTableStore);
  }

  @Test
  public void needsReindex_groupRepositoryAlwaysFalse() {
    when(repository.getType()).thenReturn(new GroupType());
    assertFalse(underTest.needsReindex(repository));
  }

  @Test
  public void needsReindex_noIndexesInStore() {
    when(repository.getName()).thenReturn("test-repo");
    when(searchTableStore.repositoryNeedsReindex(repository.getName())).thenReturn(true);

    assertTrue(underTest.needsReindex(repository));
  }

  @Test
  public void needsReindex_indexesExistInStore() {
    when(repository.getName()).thenReturn("test-repo");
    when(searchTableStore.repositoryNeedsReindex(repository.getName())).thenReturn(false);

    assertFalse(underTest.needsReindex(repository));
  }

  @Test
  public void needsReindex_byFlag() {
    attributes.set(SEARCH_INDEX_OUTDATED, true);

    assertTrue(underTest.needsReindex(repository));
  }
}
