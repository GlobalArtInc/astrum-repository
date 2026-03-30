
package org.globalart.astrum.repository.maven.tasks;

import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.maven.MavenFacet;
import org.globalart.astrum.repository.maven.internal.Maven2Format;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.CoreMatchers.is;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.maven.VersionPolicy.MIXED;
import static org.globalart.astrum.repository.maven.VersionPolicy.RELEASE;
import static org.globalart.astrum.repository.maven.VersionPolicy.SNAPSHOT;

@RunWith(MockitoJUnitRunner.Silent.class)
public class PurgeMavenUnusedSnapshotsTaskTest

{
  @Mock
  private Repository repository;

  @Mock
  private MavenFacet mavenFacet;

  private final Type groupType = new GroupType();

  private final Type hostedType = new HostedType();

  private final Format maven2Format = new Maven2Format();

  @Mock
  private Format dockerFormat;

  private final PurgeMavenUnusedSnapshotsTask underTest =
      new PurgeMavenUnusedSnapshotsTask(groupType, hostedType, maven2Format);

  @Before
  public void setup() {
    when(repository.facet(MavenFacet.class)).thenReturn(mavenFacet);
  }

  @Test
  public void appliesToMavenHostedSnapshot() {
    when(repository.getFormat()).thenReturn(maven2Format);
    when(repository.getType()).thenReturn(hostedType);
    when(mavenFacet.getVersionPolicy()).thenReturn(SNAPSHOT);
    assertThat(underTest.appliesTo(repository), is(true));
  }

  @Test
  public void appliesToMavenGroupSnapshot() {
    when(repository.getFormat()).thenReturn(maven2Format);
    when(repository.getType()).thenReturn(groupType);
    when(mavenFacet.getVersionPolicy()).thenReturn(SNAPSHOT);
    assertThat(underTest.appliesTo(repository), is(true));
  }

  @Test
  public void appliesToMavenHostedMixed() {
    when(repository.getFormat()).thenReturn(maven2Format);
    when(repository.getType()).thenReturn(hostedType);
    when(mavenFacet.getVersionPolicy()).thenReturn(MIXED);
    assertThat(underTest.appliesTo(repository), is(true));
  }

  @Test
  public void appliesToMavenHostedNoVersionPolicy() {
    when(repository.getFormat()).thenReturn(maven2Format);
    when(repository.getType()).thenReturn(hostedType);
    when(mavenFacet.getVersionPolicy()).thenReturn(null);
    assertThat(underTest.appliesTo(repository), is(true));
  }

  @Test
  public void doesNotApplyToDockerGroupSnapshot() {
    when(repository.getFormat()).thenReturn(dockerFormat);
    when(repository.getType()).thenReturn(groupType);
    when(mavenFacet.getVersionPolicy()).thenReturn(SNAPSHOT);
    assertThat(underTest.appliesTo(repository), is(false));
  }

  @Test
  public void doesNotApplyToMavenHostedRelease() {
    when(repository.getFormat()).thenReturn(maven2Format);
    when(repository.getType()).thenReturn(hostedType);
    when(mavenFacet.getVersionPolicy()).thenReturn(RELEASE);
    assertThat(underTest.appliesTo(repository), is(false));
  }
}
