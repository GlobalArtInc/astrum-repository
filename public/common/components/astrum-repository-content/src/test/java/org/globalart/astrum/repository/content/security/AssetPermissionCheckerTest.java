
package org.globalart.astrum.repository.content.security;

import java.util.Collections;
import java.util.List;
import java.util.Map.Entry;
import java.util.Optional;
import java.util.Set;
import java.util.stream.Collectors;
import java.util.stream.Stream;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetFinder;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.ContentPermissionChecker;
import org.globalart.astrum.repository.security.VariableResolverAdapter;
import org.globalart.astrum.repository.security.VariableResolverAdapterManager;
import org.globalart.astrum.selector.VariableSource;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.ArgumentMatchers.anyString;
import static org.mockito.ArgumentMatchers.eq;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class AssetPermissionCheckerTest
{
  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private ContentFacetFinder contentFacetFinder;

  @Mock
  private ContentPermissionChecker contentPermissionChecker;

  @Mock
  private VariableResolverAdapterManager variableResolverAdapterManager;

  @Mock
  private VariableResolverAdapter variableResolverAdapter;

  @Mock
  private Repository repository;

  @Mock
  private VariableSource variableSource;

  private AssetPermissionChecker underTest;

  @Before
  public void setUp() {
    when(variableResolverAdapterManager.get(anyString())).thenReturn(variableResolverAdapter);
    when(variableResolverAdapter.fromPath(anyString(), anyString())).thenReturn(variableSource);
    when(repository.getName()).thenReturn("test-repo");

    underTest = new AssetPermissionChecker(
        repositoryManager, contentFacetFinder, contentPermissionChecker, variableResolverAdapterManager);
  }

  @Test
  public void testFindPermittedAssetsReturnsEmptyForEmptyCollection() {
    Stream<Entry<Asset, String>> result = underTest.findPermittedAssets(Collections.emptyList(), "maven2", "read");
    assertThat(result.count(), is(0L));
  }

  @Test
  public void testFindPermittedAssetsReturnsPermittedAssets() {
    Asset asset = mock(Asset.class);
    when(asset.path()).thenReturn("/com/example/artifact-1.0.jar");

    when(contentFacetFinder.findRepository(eq("maven2"), any())).thenReturn(Optional.of(repository));
    when(repositoryManager.findContainingGroups("test-repo")).thenReturn(Set.of());
    when(contentPermissionChecker.isPermitted(eq("test-repo"), eq("maven2"), eq("read"), any()))
        .thenReturn(true);

    List<Entry<Asset, String>> result =
        underTest.findPermittedAssets(Collections.singletonList(asset), "maven2", "read")
            .collect(Collectors.toList());

    assertThat(result.size(), is(1));
    assertThat(result.get(0).getKey(), is(asset));
    assertThat(result.get(0).getValue(), is("test-repo"));
  }

  @Test
  public void testFindPermittedAssetsFiltersOutDeniedAssets() {
    Asset asset = mock(Asset.class);
    when(asset.path()).thenReturn("/com/example/artifact-1.0.jar");

    when(contentFacetFinder.findRepository(eq("maven2"), any())).thenReturn(Optional.of(repository));
    when(repositoryManager.findContainingGroups("test-repo")).thenReturn(Set.of());
    when(contentPermissionChecker.isPermitted(anyString(), anyString(), anyString(), any()))
        .thenReturn(false);

    List<Entry<Asset, String>> result =
        underTest.findPermittedAssets(Collections.singletonList(asset), "maven2", "read")
            .collect(Collectors.toList());

    assertThat(result.size(), is(0));
  }

  @Test
  public void testFindPermittedAssetsWithNoRepository() {
    Asset asset = mock(Asset.class);
    when(asset.path()).thenReturn("/some/path");

    when(contentFacetFinder.findRepository(anyString(), any())).thenReturn(Optional.empty());

    List<Entry<Asset, String>> result =
        underTest.findPermittedAssets(Collections.singletonList(asset), "maven2", "read")
            .collect(Collectors.toList());

    assertThat(result.size(), is(0));
  }

  @Test
  public void testIsPermittedReturnsTrueWhenPermitted() {
    Asset asset = mock(Asset.class);
    when(asset.path()).thenReturn("/path/to/asset");

    when(contentFacetFinder.findRepository(eq("maven2"), any())).thenReturn(Optional.of(repository));
    when(repositoryManager.findContainingGroups("test-repo")).thenReturn(Set.of());
    when(contentPermissionChecker.isPermitted(eq("test-repo"), eq("maven2"), eq("read"), any()))
        .thenReturn(true);

    Optional<String> result = underTest.isPermitted(asset, "maven2", "read");
    assertThat(result.isPresent(), is(true));
    assertThat(result.get(), is("test-repo"));
  }

  @Test
  public void testIsPermittedReturnsFalseWhenDenied() {
    Asset asset = mock(Asset.class);
    when(asset.path()).thenReturn("/path/to/asset");

    when(contentFacetFinder.findRepository(eq("maven2"), any())).thenReturn(Optional.of(repository));
    when(repositoryManager.findContainingGroups("test-repo")).thenReturn(Set.of());
    when(contentPermissionChecker.isPermitted(anyString(), anyString(), anyString(), any()))
        .thenReturn(false);

    Optional<String> result = underTest.isPermitted(asset, "maven2", "read");
    assertThat(result.isPresent(), is(false));
  }

  @Test
  public void testIsPermittedChecksGroupRepositories() {
    Asset asset = mock(Asset.class);
    when(asset.path()).thenReturn("/path/to/asset");

    when(contentFacetFinder.findRepository(eq("maven2"), any())).thenReturn(Optional.of(repository));
    Set<String> groups = Set.of("maven-group");
    when(repositoryManager.findContainingGroups("test-repo")).thenReturn(groups);

    // Deny direct repo but allow group
    when(contentPermissionChecker.isPermitted(eq("test-repo"), eq("maven2"), eq("read"), any()))
        .thenReturn(false);
    when(contentPermissionChecker.isPermitted(eq("maven-group"), eq("maven2"), eq("read"), any()))
        .thenReturn(true);

    Optional<String> result = underTest.isPermitted(asset, "maven2", "read");
    assertThat(result.isPresent(), is(true));
    assertThat(result.get(), is("maven-group"));
  }

  @Test
  public void testIsPermittedReturnsEmptyWhenNoRepository() {
    Asset asset = mock(Asset.class);
    when(asset.path()).thenReturn("/path/to/asset");

    when(contentFacetFinder.findRepository(anyString(), any())).thenReturn(Optional.empty());

    Optional<String> result = underTest.isPermitted(asset, "maven2", "read");
    assertThat(result.isPresent(), is(false));
  }

  @Test(expected = NullPointerException.class)
  public void testNullRepositoryManagerRejected() {
    new AssetPermissionChecker(null, contentFacetFinder, contentPermissionChecker, variableResolverAdapterManager);
  }

  @Test(expected = NullPointerException.class)
  public void testNullContentFacetFinderRejected() {
    new AssetPermissionChecker(repositoryManager, null, contentPermissionChecker, variableResolverAdapterManager);
  }
}
