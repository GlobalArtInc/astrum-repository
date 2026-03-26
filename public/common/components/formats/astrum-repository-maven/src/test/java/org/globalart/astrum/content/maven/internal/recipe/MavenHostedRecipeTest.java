
package org.globalart.astrum.content.maven.internal.recipe;

import jakarta.inject.Provider;

import org.globalart.astrum.content.maven.internal.index.MavenContentHostedIndexFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.maven.PurgeUnusedSnapshotsFacet;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.types.HostedType;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.Mockito.verify;

public class MavenHostedRecipeTest
    extends MavenRecipeTestSupport
{
  @Mock
  private Repository mavenHostedRepository;

  @Mock
  private MavenContentHostedIndexFacet mavenHostedIndexFacet;

  private final Provider<MavenContentHostedIndexFacet> mavenHostedIndexFacetProvider = () -> mavenHostedIndexFacet;

  @Mock
  private PurgeUnusedSnapshotsFacet purgeUnusedSnapshotsFacet;

  private final Provider<PurgeUnusedSnapshotsFacet> mavenPurgeUnusedSnapshotsFacetProvider =
      () -> purgeUnusedSnapshotsFacet;

  private MavenHostedRecipe underTest;

  @Before
  public void setup() {
    underTest = new MavenHostedRecipe(new HostedType(), new Maven2Format(), mavenHostedIndexFacetProvider,
        mavenPurgeUnusedSnapshotsFacetProvider);
    mockFacets(underTest);
    mockHandlers(underTest);
  }

  @Test
  public void testExpectedFacetsAreAttached() throws Exception {
    underTest.apply(mavenHostedRepository);
    verify(mavenHostedRepository).attach(securityFacet);
    verify(mavenHostedRepository).attach(viewFacet);
    verify(mavenHostedRepository).attach(mavenMetadataRebuildFacet);
    verify(mavenHostedRepository).attach(mavenContentFacet);
    verify(mavenHostedRepository).attach(searchFacet);
    verify(mavenHostedRepository).attach(browseFacet);
    verify(mavenHostedRepository).attach(mavenArchetypeCatalogFacet);
    verify(mavenHostedRepository).attach(mavenHostedIndexFacet);
    verify(mavenHostedRepository).attach(mavenMaintenanceFacet);
    verify(mavenHostedRepository).attach(removeSnapshotsFacet);
    verify(mavenHostedRepository).attach(purgeUnusedSnapshotsFacet);
  }
}
