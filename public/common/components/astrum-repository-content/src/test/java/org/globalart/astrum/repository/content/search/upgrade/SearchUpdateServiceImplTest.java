
package org.globalart.astrum.repository.content.search.upgrade;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.search.index.SearchUpdateService.SEARCH_INDEX_OUTDATED;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SearchUpdateServiceImplTest
{
  @Mock
  private Repository repository;

  @Mock
  private ContentFacet contentFacet;

  private final NestedAttributesMap attributes = new NestedAttributesMap();

  private final SearchUpdateServiceImpl underTest = new SearchUpdateServiceImpl();

  @Before
  public void setup() {
    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(repository.getType()).thenReturn(new HostedType());
    when(contentFacet.attributes()).thenReturn(attributes);
  }

  @Test
  public void needsReindex_groupRepositoryAlwaysFalse() {
    when(repository.getType()).thenReturn(new GroupType());
    assertFalse(underTest.needsReindex(repository));
  }

  @Test
  public void needsReindex_noFlag() {
    assertFalse(underTest.needsReindex(repository));
  }

  @Test
  public void needsReindex_invalidFlag() {
    attributes.set(SEARCH_INDEX_OUTDATED, "hello");
    assertFalse(underTest.needsReindex(repository));
  }

  @Test
  public void needsReindex_falseFlag() {
    attributes.set(SEARCH_INDEX_OUTDATED, false);
    assertFalse(underTest.needsReindex(repository));
  }

  @Test
  public void needsReindex_trueFlag() {
    attributes.set(SEARCH_INDEX_OUTDATED, true);
    assertTrue(underTest.needsReindex(repository));
  }
}
