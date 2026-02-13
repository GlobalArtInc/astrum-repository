
package org.globalart.astrum.repository.content.upgrades;

import java.util.Optional;

import org.junit.Before;
import org.junit.Test;

import static org.assertj.core.api.Assertions.assertThat;

public class SoftDeletedBlobsByBlobStoreIndexMigrationStep_2_7Test
{
  private SoftDeletedBlobsByBlobStoreIndexMigrationStep_2_7 underTest;

  @Before
  public void setup() {
    underTest = new SoftDeletedBlobsByBlobStoreIndexMigrationStep_2_7();
  }

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();

    assertThat(version).isPresent();
    assertThat(version.get()).isEqualTo("2.7");
  }

  @Test
  public void testGetChecksum() {
    assertThat(underTest.getChecksum()).isNotNull();
  }
}
