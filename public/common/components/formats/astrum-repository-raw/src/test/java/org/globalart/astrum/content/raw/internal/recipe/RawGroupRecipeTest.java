
package org.globalart.astrum.content.raw.internal.recipe;

import jakarta.inject.Provider;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.types.GroupType;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.Mockito.verify;

public class RawGroupRecipeTest
    extends RawRecipeTestSupport
{
  @Mock
  private Repository rawGroupRepository;

  @Mock
  private GroupHandler groupHandler;

  @Mock
  private GroupFacet groupFacet;

  private final Provider<GroupFacet> groupFacetProvider = () -> groupFacet;

  private RawGroupRecipe underTest;

  @Before
  public void setup() {
    underTest = new RawGroupRecipe(new GroupType(), new RawFormat(), groupFacetProvider, groupHandler);
    mockDependencies(underTest);
  }

  @Test
  public void testExpectedFacetsAreAttached() throws Exception {
    underTest.apply(rawGroupRepository);
    verify(rawGroupRepository).attach(securityFacet);
    verify(rawGroupRepository).attach(viewFacet);
    verify(rawGroupRepository).attach(groupFacet);
    verify(rawGroupRepository).attach(contentFacet);
    verify(rawGroupRepository).attach(browseFacet);
  }
}
