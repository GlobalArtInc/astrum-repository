
package org.globalart.astrum.upgrade.datastore.internal.steps;

import java.sql.Connection;
import java.util.Optional;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.mockito.Mockito.verifyNoInteractions;

/**
 * Availability-marker step for HuggingFace XET per-repository config attributes (§4 Phase 2
 * Task 2 of xet-implementation-spec). No schema change is required — the two attributes
 * ({@code xetCasUrl}, {@code xetTransferHostAllowList}) live inside the shared
 * {@code repository.attributes} JSON column, so the step is a pure no-op that only exists to
 * advance the Flyway version far enough for {@code @AvailabilityVersion(from = "2.162")} on the
 * HuggingFace proxy recipe to activate.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class HuggingFaceProxyDatabaseMigrationStep_2_162Test
{
  @Mock
  private Connection connection;

  private HuggingFaceProxyDatabaseMigrationStep_2_162 underTest;

  @Before
  public void setup() {
    underTest = new HuggingFaceProxyDatabaseMigrationStep_2_162();
  }

  @Test
  public void versionIs_2_162() {
    assertThat(underTest.version(), is(Optional.of("2.162")));
  }

  @Test
  public void migrateIsNoOp() throws Exception {
    underTest.migrate(connection);
    verifyNoInteractions(connection);
  }
}
