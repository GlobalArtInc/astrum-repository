
package org.globalart.astrum.content.maven.internal.recipe;

import jakarta.inject.Provider;

import org.globalart.astrum.content.maven.internal.index.MavenContentGroupIndexFacet;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.maven.PurgeUnusedSnapshotsFacet;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.maven.internal.group.MavenGroupFacet;
import org.globalart.astrum.repository.maven.internal.group.MergingGroupHandler;
import org.globalart.astrum.repository.types.GroupType;

import org.junit.Before;
import org.junit.Test;
import org.mockito.Mock;

import static org.mockito.Mockito.verify;

public class MavenGroupRecipeTest
    extends MavenRecipeTestSupport
{
  @Mock
  private Repository mavenGroupRepository;

  @Mock
  private MavenGroupFacet mavenGroupFacet;

  private final Provider<MavenGroupFacet> mavenGroupFacetProvider = () -> mavenGroupFacet;

  @Mock
  private MavenContentGroupIndexFacet mavenContentIndexFacet;

  private final Provider<MavenContentGroupIndexFacet> mavenContentIndexFacetProvider = () -> mavenContentIndexFacet;

  @Mock
  private PurgeUnusedSnapshotsFacet purgeUnusedSnapshotsFacet;

  private final Provider<PurgeUnusedSnapshotsFacet> purgeUnusedSnapshotsFacetProvider = () -> purgeUnusedSnapshotsFacet;

  @Mock
  private MergingGroupHandler mergingGroupHandler;

  @Mock
  private MavenContentIndexGroupHandler mavenContentIndexGroupHandler;

  @Mock
  private GroupHandler groupHandler;

  private MavenGroupRecipe underTest;

  @Before
  public void setup() {
    underTest = new MavenGroupRecipe(new GroupType(), new Maven2Format(), mavenContentIndexFacetProvider,
        mavenGroupFacetProvider, purgeUnusedSnapshotsFacetProvider, groupHandler, mergingGroupHandler,
        mavenContentIndexGroupHandler);
    mockFacets(underTest);
    mockHandlers(underTest);
  }

  @Test
  public void testExpectedFacetsAreAttached() throws Exception {
    underTest.apply(mavenGroupRepository);
    verify(mavenGroupRepository).attach(securityFacet);
    verify(mavenGroupRepository).attach(mavenGroupFacet);
    verify(mavenGroupRepository).attach(mavenContentFacet);
    verify(mavenGroupRepository).attach(mavenContentIndexFacet);
    verify(mavenGroupRepository).attach(browseFacet);
    verify(mavenGroupRepository).attach(purgeUnusedSnapshotsFacet);
    verify(mavenGroupRepository).attach(viewFacet);
    verify(mavenGroupRepository).attach(mavenMaintenanceFacet);
    verify(mavenGroupRepository).attach(removeSnapshotsFacet);
  }

}
