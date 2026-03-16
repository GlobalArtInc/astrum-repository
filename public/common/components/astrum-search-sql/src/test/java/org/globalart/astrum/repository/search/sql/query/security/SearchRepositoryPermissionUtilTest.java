
package org.globalart.astrum.repository.search.sql.query.security;

import java.util.Collections;
import java.util.List;
import java.util.Set;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.security.RepositoryViewPermission;
import org.globalart.astrum.security.SecurityHelper;
import org.globalart.astrum.selector.SelectorConfiguration;
import org.globalart.astrum.selector.SelectorManager;

import com.google.common.collect.ImmutableMap;
import com.google.common.collect.ImmutableSet;
import org.junit.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;

import static java.util.Arrays.asList;
import static java.util.Collections.emptySet;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsInAnyOrder;
import static org.hamcrest.core.Is.is;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.security.BreadActions.BROWSE;
import static org.globalart.astrum.selector.SelectorConfiguration.EXPRESSION;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class SearchRepositoryPermissionUtilTest
{
  private static final String SELECTOR_EXPRESSION_1 = "format=\"raw\" and path^=\"/awesome-raw-file.txt\"";

  private static final String SELECTOR_EXPRESSION_2 = "format=\"raw\" and path^=\"/super-raw-file.txt\"";

  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private SecurityHelper securityHelper;

  @Mock
  private SelectorManager selectorManager;

  @Mock
  private SelectorConfiguration selectorConfiguration1;

  @Mock
  private SelectorConfiguration selectorConfiguration2;

  @Mock
  private SelectorConfiguration selectorConfiguration3;

  @InjectMocks
  private SearchRepositoryPermissionUtil underTest;

  @Test
  public void shouldReturnUnknownRepositories() {
    Set<String> repositories = ImmutableSet.of("repo1", "repo2", "repo3", "repo4");

    Set<String> unknownRepositories = underTest.browsableAndUnknownRepositories(repositories);

    assertThat(unknownRepositories, containsInAnyOrder(repositories.toArray(new String[0])));
    verify(securityHelper, never()).allPermitted(any());
  }

  @Test
  public void shouldReturnBrowsableRepositories() {
    Set<String> repositories = ImmutableSet.of("repo1", "repo2", "repo3", "repo4");
    Set<String> expected = ImmutableSet.of("repo2", "repo3");

    mockRepositoryExists(repositories);
    mockRepositoryBrowsePermission(expected);

    Set<String> browsableRepositories = underTest.browsableAndUnknownRepositories(repositories);

    assertThat(browsableRepositories, containsInAnyOrder(expected.toArray(new String[0])));
  }

  @Test
  public void selectorConfigurationsShouldBeEmptyList() {
    List<SelectorConfiguration> selectorConfigurations = underTest.selectorConfigurations(emptySet());

    assertThat(selectorConfigurations.isEmpty(), is(true));
  }

  @Test
  public void shouldGetDistinctSelectorConfigurations() {
    Set<String> repositories = ImmutableSet.of("repo2", "repo3");
    mockSelectorConfigurations(repositories);

    List<SelectorConfiguration> selectorConfigurations = underTest.selectorConfigurations(repositories);

    assertThat(selectorConfigurations, containsInAnyOrder(selectorConfiguration1, selectorConfiguration2));
  }

  private List<SelectorConfiguration> getSelectorConfigurations() {
    return asList(selectorConfiguration1, selectorConfiguration2, selectorConfiguration3);
  }

  private void mockSelectorConfigurations(final Set<String> repositories) {
    when(selectorConfiguration1.getAttributes()).thenReturn(ImmutableMap.of(EXPRESSION, SELECTOR_EXPRESSION_1));
    when(selectorConfiguration2.getAttributes()).thenReturn(ImmutableMap.of(EXPRESSION, SELECTOR_EXPRESSION_2));
    when(selectorConfiguration3.getAttributes()).thenReturn(ImmutableMap.of(EXPRESSION, SELECTOR_EXPRESSION_2));

    when(selectorManager.browseActive(repositories, Collections.emptyList())).thenReturn(getSelectorConfigurations());
  }

  private void mockRepositoryBrowsePermission(final Set<String> repositories) {
    repositories.forEach(
        repository -> when(securityHelper.allPermitted(new RepositoryViewPermission("raw", repository, BROWSE)))
            .thenReturn(true));
  }

  private void mockRepositoryExists(final Set<String> repositories) {
    Format format = mock(Format.class);
    when(format.getValue()).thenReturn("raw");
    repositories.forEach(repository -> {
      Repository repo = mock(Repository.class);
      when(repo.getName()).thenReturn(repository);
      when(repo.getFormat()).thenReturn(format);
      when(repositoryManager.get(repository)).thenReturn(repo);
      when(repositoryManager.exists(repository)).thenReturn(true);
    });
  }
}
