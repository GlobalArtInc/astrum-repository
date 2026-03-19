
package org.globalart.astrum.content.raw.internal.recipe;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.raw.internal.RawFormat;
import org.globalart.astrum.repository.types.HostedType;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.Mockito.verify;

public class RawHostedRecipeTest
    extends RawRecipeTestSupport
{
  @Mock
  private Repository rawHostedRepository;

  private RawHostedRecipe underTest;

  @Before
  public void setup() {
    underTest = new RawHostedRecipe(new HostedType(), new RawFormat());
    mockDependencies(underTest);
  }

  @Test
  public void testExpectedFacetsAreAttached() throws Exception {
    underTest.apply(rawHostedRepository);
    verify(rawHostedRepository).attach(securityFacet);
    verify(rawHostedRepository).attach(viewFacet);
    verify(rawHostedRepository).attach(contentFacet);
    verify(rawHostedRepository).attach(maintenanceFacet);
    verify(rawHostedRepository).attach(searchFacet);
    verify(rawHostedRepository).attach(browseFacet);
  }

}
