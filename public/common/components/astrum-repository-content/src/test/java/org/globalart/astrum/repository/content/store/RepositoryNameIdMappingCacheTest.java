
package org.globalart.astrum.repository.content.store;

import java.util.Arrays;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.manager.RepositoryCreatedEvent;
import org.globalart.astrum.repository.manager.RepositoryDeletedEvent;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.CoreMatchers.not;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.hasEntry;
import static org.mockito.ArgumentMatchers.any;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RepositoryNameIdMappingCacheTest
{
  @Mock
  private FormatStoreManager formatStoreManager;

  @Mock
  private List<Format> formatNames;

  @Mock
  private ContentRepositoryStore<?> contentRepositoryStore;

  private RepositoryNameIdMappingCache underTest;

  @Mock
  private DatabaseCheck databaseCheck;

  @Before
  public void setup() {
    when(formatStoreManager.contentRepositoryStore(any())).thenReturn(contentRepositoryStore);
    when(databaseCheck.isPostgresql()).thenReturn(true);
    underTest = new RepositoryNameIdMappingCache(List.of(formatStoreManager), formatNames, databaseCheck);
  }

  @Test
  public void testGetRepositoryIds() {
    Map<Integer, String> repositoryIds = initializeCache();
    assertThat(repositoryIds.keySet(), contains(1, 2));
    assertThat(repositoryIds, hasEntry(1, "repo1"));
    assertThat(repositoryIds, hasEntry(2, "repo2"));
  }

  @Test
  public void testShouldAddNewRepositoryIdOnRepositoryCreatedEvent() {
    initializeCache();
    Map<String, Object> nameIdThree = getRepositoryNameId("repo3", 3);

    when(contentRepositoryStore.readContentRepositoryId(any(), any())).thenReturn(Optional.of(nameIdThree));

    underTest.on(new RepositoryCreatedEvent(repository("repo3")));

    Map<Integer, String> repositoryIds = underTest.getRepositoryNameIds(Arrays.asList("repo3"), "raw");
    assertThat(repositoryIds, hasEntry(3, "repo3"));
  }

  @Test
  public void testShouldRemoveRepositoryIdOnRepositoryDeletedEvent() {
    initializeCache();

    underTest.on(new RepositoryDeletedEvent(repository("repo1")));

    Map<Integer, String> repositoryIds = underTest.getRepositoryNameIds(Arrays.asList("repo1"), "raw");
    assertThat(repositoryIds, not(hasEntry(2, "repo2")));
  }

  private Map<Integer, String> initializeCache() {
    List<String> repositoryNames = Arrays.asList("repo1", "repo2");
    Map<String, Object> nameIdOne = getRepositoryNameId("repo1", 1);
    Map<String, Object> nameIdTwo = getRepositoryNameId("repo2", 2);
    List<Map<String, Object>> repositoryNameIds = Arrays.asList(nameIdOne, nameIdTwo);

    when(contentRepositoryStore.readAllContentRepositoryIds(any())).thenReturn(repositoryNameIds);

    return underTest.getRepositoryNameIds(repositoryNames, "raw");
  }

  private Map<String, Object> getRepositoryNameId(String name, int id) {
    Map<String, Object> nameId = new HashMap<>();
    nameId.put("name", name);
    nameId.put("repository_id", id);
    return nameId;
  }

  private Repository repository(String name) {
    Repository repository = mock(Repository.class);
    when(repository.getName()).thenReturn(name);
    when(repository.getName()).thenReturn(name);
    when(repository.getFormat()).thenReturn(mock(Format.class));
    return repository;
  }
}
