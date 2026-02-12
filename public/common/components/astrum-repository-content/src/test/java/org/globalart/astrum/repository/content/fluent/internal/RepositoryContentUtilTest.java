
package org.globalart.astrum.repository.content.fluent.internal;

import java.util.HashSet;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Arrays.asList;
import static java.util.Collections.singletonList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.junit.Assert.assertFalse;
import static org.junit.Assert.assertTrue;
import static org.mockito.Mockito.never;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.content.fluent.internal.RepositoryContentUtil.getRepositoryIds;
import static org.globalart.astrum.repository.content.fluent.internal.RepositoryContentUtil.isGroupRepository;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RepositoryContentUtilTest
{
  @Mock
  private Repository repository;

  @Mock
  private Repository leafMember1;

  @Mock
  private Repository leafMember2;

  @Mock
  private ContentFacet contentFacet;

  @Mock
  private GroupFacet groupFacet;

  @Mock
  private ContentFacet member1ContentFacet;

  @Mock
  private ContentFacet member2ContentFacet;

  @Before
  public void setup() {
    when(repository.getType()).thenReturn(new HostedType());
    when(repository.facet(ContentFacet.class)).thenReturn(contentFacet);
    when(repository.facet(GroupFacet.class)).thenReturn(groupFacet);
    when(leafMember1.facet(ContentFacet.class)).thenReturn(member1ContentFacet);
    when(leafMember2.facet(ContentFacet.class)).thenReturn(member2ContentFacet);
    when(groupFacet.leafMembers()).thenReturn(asList(leafMember1, leafMember2));
    when(contentFacet.contentRepositoryId()).thenReturn(1);
    when(member1ContentFacet.contentRepositoryId()).thenReturn(2);
    when(member2ContentFacet.contentRepositoryId()).thenReturn(3);
  }

  @Test
  public void testIsGroupRepository_hostedRepository() {
    assertFalse(isGroupRepository(repository));
  }

  @Test
  public void testIsGroupRepository_groupRepository() {
    when(repository.getType()).thenReturn(new GroupType());

    assertTrue(isGroupRepository(repository));
  }

  @Test
  public void testGetRepositoryIds_hostedRepository() {
    assertThat(getRepositoryIds(null, contentFacet, repository), contains(1));
  }

  @Test
  public void testGetRepositoryIds_groupRepository() {
    when(repository.getType()).thenReturn(new GroupType());

    assertThat(getRepositoryIds(null, contentFacet, repository), contains(1));
  }

  @Test
  public void testGetRepositoryIds_constraintOverridesHosted() {
    assertThat(
        getRepositoryIds(singletonList(repository -> new HashSet<>(asList(1, 2, 3, 4))), contentFacet, repository),
        contains(1, 2, 3, 4));

    // repositoryId should be solely coming from the constraint
    verify(contentFacet, never()).contentRepositoryId();
    verifyNoInteractions(contentFacet);
  }

  @Test
  public void testGetRepositoryIds_constraintOverridesGroup() {
    when(repository.getType()).thenReturn(new GroupType());

    assertThat(
        getRepositoryIds(singletonList(repository -> new HashSet<>(asList(1, 2, 3, 4))), contentFacet, repository),
        contains(1, 2, 3, 4));

    // repositoryId should be solely coming from the constraint
    verify(contentFacet, never()).contentRepositoryId();
    verifyNoInteractions(contentFacet);
  }
}
