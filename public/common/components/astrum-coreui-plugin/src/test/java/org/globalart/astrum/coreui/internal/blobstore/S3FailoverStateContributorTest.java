
package org.globalart.astrum.coreui.internal.blobstore;

import org.globalart.astrum.common.db.DatabaseCheck;

import org.junit.Test;
import org.mockito.Mock;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.hasEntry;
import static org.mockito.Mockito.verify;
import static org.mockito.Mockito.verifyNoInteractions;
import static org.mockito.Mockito.verifyNoMoreInteractions;
import static org.mockito.Mockito.when;
import org.junit.runner.RunWith;
import org.mockito.junit.MockitoJUnitRunner;

@RunWith(MockitoJUnitRunner.Silent.class)
public class S3FailoverStateContributorTest
{
  @Mock
  private DatabaseCheck databaseCheck;

  private S3FailoverStateContributor underTest;

  @Test
  public void failoverAvailableNotZdu() {
    underTest = new S3FailoverStateContributor(databaseCheck, false);
    assertThat(underTest.getState(), hasEntry("S3FailoverEnabled", true));
    verifyNoInteractions(databaseCheck);
  }

  @Test
  public void failoverNotAvailableZduAndNotInVersion() {
    when(databaseCheck.isAtLeast("2.6")).thenReturn(false);
    underTest = new S3FailoverStateContributor(databaseCheck, true);
    assertThat(underTest.getState(), hasEntry("S3FailoverEnabled", false));
    verify(databaseCheck).isAtLeast("2.6");
    verifyNoMoreInteractions(databaseCheck);
  }

  @Test
  public void failoverAvailableZduAndMinimumVersion() {
    when(databaseCheck.isAtLeast("2.6")).thenReturn(true);
    underTest = new S3FailoverStateContributor(databaseCheck, true);
    assertThat(underTest.getState(), hasEntry("S3FailoverEnabled", true));
    verify(databaseCheck).isAtLeast("2.6");
    verifyNoMoreInteractions(databaseCheck);
  }
}
