
package org.globalart.astrum.repository.content.fluent.constraints;

import java.util.Set;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.group.GroupFacet;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static java.util.Arrays.asList;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.content.fluent.constraints.GroupRepositoryConstraint.GroupRepositoryLocation.BOTH;
import static org.globalart.astrum.repository.content.fluent.constraints.GroupRepositoryConstraint.GroupRepositoryLocation.LOCAL;
import static org.globalart.astrum.repository.content.fluent.constraints.GroupRepositoryConstraint.GroupRepositoryLocation.MEMBERS;

@RunWith(MockitoJUnitRunner.Silent.class)
public class GroupRepositoryConstraintTest
{
  @Mock
  private Repository repository;

  @Mock
  private Repository leafMember1;

  @Mock
  private Repository leafMember2;

  @Mock
  private GroupFacet groupFacet;

  @Mock
  private ContentFacet groupContentFacet;

  @Mock
  private ContentFacet member1ContentFacet;

  @Mock
  private ContentFacet member2ContentFacet;

  private GroupRepositoryConstraint underTest;

  @Before
  public void setup() {
    when(repository.facet(GroupFacet.class)).thenReturn(groupFacet);
    when(repository.facet(ContentFacet.class)).thenReturn(groupContentFacet);
    when(groupFacet.leafMembers()).thenReturn(asList(leafMember1, leafMember2));
    when(leafMember1.facet(ContentFacet.class)).thenReturn(member1ContentFacet);
    when(leafMember2.facet(ContentFacet.class)).thenReturn(member2ContentFacet);
    when(groupContentFacet.contentRepositoryId()).thenReturn(1);
    when(member1ContentFacet.contentRepositoryId()).thenReturn(2);
    when(member2ContentFacet.contentRepositoryId()).thenReturn(3);
  }

  @Test
  public void testGetRepositoryIds_local() {
    underTest = new GroupRepositoryConstraint(LOCAL);
    Set<Integer> repositoryIds = underTest.getRepositoryIds(repository);

    assertThat(repositoryIds, contains(1));
  }

  @Test
  public void testGetRepositoryIds_members() {
    underTest = new GroupRepositoryConstraint(MEMBERS);
    Set<Integer> repositoryIds = underTest.getRepositoryIds(repository);

    assertThat(repositoryIds, contains(2, 3));
  }

  @Test
  public void testGetRepositoryIds_both() {
    underTest = new GroupRepositoryConstraint(BOTH);
    Set<Integer> repositoryIds = underTest.getRepositoryIds(repository);

    assertThat(repositoryIds, contains(1, 2, 3));
  }
}
