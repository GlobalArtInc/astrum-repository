
package org.globalart.astrum.script.plugin.internal.provisioning;

import org.junit.Test;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import com.google.common.collect.ImmutableMap;

import org.globalart.astrum.blobstore.MockBlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStore;
import org.globalart.astrum.blobstore.api.BlobStoreConfiguration;
import org.globalart.astrum.blobstore.api.BlobStoreManager;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.RepositoryManager;

import java.util.Collections;
import java.util.Map;

import static org.mockito.Mockito.*;
import static org.junit.Assert.*;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RepositoryApiImplTest
{
  @Mock
  private RepositoryManager repositoryManager;

  @Mock
  private BlobStoreManager blobStoreManager;

  @InjectMocks
  private RepositoryApiImpl api;

  @Test(expected = IllegalArgumentException.class)
  public void testCannotValidateBlobStoreThatDoesNotExist() {
    when(blobStoreManager.browse()).thenReturn(Collections.emptyList());
    api.validateBlobStore(configWithAttributes(ImmutableMap.of("storage", ImmutableMap.of("blobStoreName", "foo"))));
  }

  @Test
  public void testCanValidateGivenAnExistingBlobStore() {
    BlobStore blobStore = mock(BlobStore.class);
    BlobStoreConfiguration configuration = new MockBlobStoreConfiguration();
    configuration.setName("foo");

    when(blobStoreManager.browse()).thenReturn(Collections.singletonList(blobStore));
    when(blobStore.getBlobStoreConfiguration()).thenReturn(configuration);

    api.validateBlobStore(configWithAttributes(ImmutableMap.of("storage", ImmutableMap.of("blobStoreName", "foo"))));

    verify(blobStoreManager).browse();
    verify(blobStore).getBlobStoreConfiguration();
  }

  @Test(expected = ClassCastException.class)
  public void testGroupMemberNamesMustBeACollection() {
    api.validateGroupMembers(configWithAttributes(ImmutableMap.of("group", ImmutableMap.of("memberNames", "foo"))));
  }

  @Test(expected = IllegalStateException.class)
  public void testCannotValidateGroupThatContainsNonExistentMembers() {
    when(repositoryManager.browse()).thenReturn(Collections.emptyList());
    api.validateGroupMembers(configWithAttributes(
        ImmutableMap.of("group", ImmutableMap.of("memberNames", Collections.singletonList("foo")))));
  }

  @Test
  public void testCanValidateGroupWithExistingMembers() {
    Repository repository = mock(Repository.class);

    when(repositoryManager.browse()).thenReturn(Collections.singletonList(repository));
    when(repository.getName()).thenReturn("foo");

    api.validateGroupMembers(configWithAttributes(
        ImmutableMap.of("group", ImmutableMap.of("memberNames", Collections.singletonList("foo")))));

    verify(repositoryManager).browse();
    verify(repository).getName();
  }

  @Test
  public void testNonGroupRepositoriesPassGroupValidationTrivially() {
    api.validateGroupMembers(configWithAttributes(Collections.emptyMap()));
    assertTrue(true);
  }

  private Configuration configWithAttributes(final Map<String, Map<String, Object>> attributes) {
    Configuration config = mock(Configuration.class);
    when(config.getAttributes()).thenReturn(attributes);
    return config;
  }
}
