
package org.globalart.astrum.content.maven.internal.recipe;

import java.time.LocalDate;
import java.util.Collections;

import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.content.maven.store.Maven2ComponentStore;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacetStores;
import org.globalart.astrum.repository.content.fluent.FluentComponents;
import org.globalart.astrum.repository.content.store.AssetStore;
import org.globalart.astrum.repository.maven.PurgeUnusedSnapshotsFacet;
import org.globalart.astrum.repository.maven.internal.group.MavenGroupFacet;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.test.util.Whitebox;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;
import org.mockito.Mockito;

import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyInt;
import static org.mockito.ArgumentMatchers.anyLong;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.times;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class PurgeUnusedSnapshotsFacetImplTest

{
  static final int FIND_UNUSED_LIMIT = 10;

  static final int NUMBER_OF_COMPONENTS = 35;

  static final int NUMBER_OF_DAYS_UNUSED = 5;

  static final Integer CONTENT_REPOSITORY_ID = 7;

  static final Integer COMPONENT_ID = 3;

  static final LocalDate OLD_SNAPSHOTS_DATE_LIMIT = LocalDate.now().minusDays(NUMBER_OF_DAYS_UNUSED);

  @Mock
  Repository repository;

  @Mock
  MavenGroupFacet mavenGroupFacet;

  @Mock
  MavenContentFacetImpl mavenContentFacet;

  @Mock
  ContentFacetStores contentFacetStores;

  @Mock
  Maven2ComponentStore componentStore;

  @Mock
  AssetStore assetStore;

  @Mock
  FluentComponents fluentComponents;

  GroupType groupType = new GroupType();

  HostedType hostedType = new HostedType();

  PurgeUnusedSnapshotsFacetImpl purgeUnusedSnapshotsFacet;

  @Before
  public void setUp() throws Exception {
    purgeUnusedSnapshotsFacet = createFacet(repository);

    when(repository.getType()).thenReturn(new HostedType());
    when(repository.facet(MavenContentFacet.class)).thenReturn(mavenContentFacet);
    when(repository.getName()).thenReturn("maven2-unit-test-hosted-repo");
    when(repository.facet(PurgeUnusedSnapshotsFacet.class)).thenReturn(purgeUnusedSnapshotsFacet);

    Whitebox.setInternalState(contentFacetStores, "componentStore", componentStore);
    Whitebox.setInternalState(contentFacetStores, "assetStore", assetStore);

    // Final methods can't be mocked so mocking fields here
    when(mavenContentFacet.stores()).thenReturn(contentFacetStores);
    when(mavenContentFacet.components()).thenReturn(fluentComponents);
    when(mavenContentFacet.contentRepositoryId()).thenReturn(CONTENT_REPOSITORY_ID);

    when(fluentComponents.count()).thenReturn(NUMBER_OF_COMPONENTS);

    when(componentStore.selectUnusedSnapshots(CONTENT_REPOSITORY_ID, OLD_SNAPSHOTS_DATE_LIMIT, FIND_UNUSED_LIMIT))
        .thenReturn(Collections.singletonList(COMPONENT_ID))
        .thenReturn(Collections.emptyList()); // Imitates deleting all components during the first iteration.
  }

  @Test
  public void deleteUnusedSnapshotComponentsInHostedRepo() {
    purgeUnusedSnapshotsFacet.purgeUnusedSnapshots(NUMBER_OF_DAYS_UNUSED);

    verify(componentStore, times(2)).selectUnusedSnapshots(anyInt(), any(), anyLong());
    verify(mavenContentFacet, times(1)).deleteComponents(new int[]{COMPONENT_ID});
  }

  @Test
  public void deleteUnusedSnapshotComponentsInGroupOfHostedRepo() throws Exception {
    Repository groupRepository = Mockito.mock(Repository.class);
    when(groupRepository.getType()).thenReturn(new GroupType());
    when(groupRepository.getName()).thenReturn("maven2-unit-test-group-repo");
    when(groupRepository.facet(MavenGroupFacet.class)).thenReturn(mavenGroupFacet);
    when(mavenGroupFacet.leafMembers()).thenReturn(Collections.singletonList(repository));

    PurgeUnusedSnapshotsFacet purgeUnusedSnapshotsFacetForGroupRepo = createFacet(groupRepository);

    purgeUnusedSnapshotsFacetForGroupRepo.purgeUnusedSnapshots(NUMBER_OF_DAYS_UNUSED);

    verify(componentStore, times(2)).selectUnusedSnapshots(anyInt(), any(), anyLong());
    verify(mavenContentFacet, times(1)).deleteComponents(new int[]{COMPONENT_ID});
  }

  private PurgeUnusedSnapshotsFacetImpl createFacet(final Repository repository) throws Exception {
    PurgeUnusedSnapshotsFacetImpl facet = new PurgeUnusedSnapshotsFacetImpl(groupType, hostedType, FIND_UNUSED_LIMIT);
    facet.installDependencies(mock(EventManager.class));
    facet.attach(repository);
    facet.init();
    facet.start();
    return facet;
  }
}
