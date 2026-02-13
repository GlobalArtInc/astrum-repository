
package org.globalart.astrum.repository.content.upgrades;

import java.util.Collections;
import java.util.List;
import java.util.Optional;

import org.globalart.astrum.repository.Format;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.assertj.core.api.Assertions.assertThat;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.Silent.class)
public class RetainNCompositeIndexMigrationStep_2_144Test
{
  @Mock
  private Format mavenFormat;

  @Mock
  private Format dockerFormat;

  @Mock
  private Format unsupportedFormat;

  private RetainNCompositeIndexMigrationStep_2_144 underTest;

  @Before
  public void setup() {
    when(mavenFormat.getValue()).thenReturn("maven2");
    when(dockerFormat.getValue()).thenReturn("docker");
    when(unsupportedFormat.getValue()).thenReturn("unsupported-format");
    underTest = new RetainNCompositeIndexMigrationStep_2_144(
        List.of(mavenFormat, dockerFormat, unsupportedFormat));
  }

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();

    assertThat(version).isPresent();
    assertThat(version.get()).isEqualTo("2.144");
  }

  @Test
  public void testCanExecuteInTransaction() {
    // Returns false because the migration uses CREATE INDEX CONCURRENTLY on PostgreSQL,
    // which cannot run inside a transaction block.
    assertThat(underTest.canExecuteInTransaction()).isFalse();
  }

  @Test
  public void testConstructorWithEmptyFormats() {
    RetainNCompositeIndexMigrationStep_2_144 step =
        new RetainNCompositeIndexMigrationStep_2_144(Collections.emptyList());

    assertThat(step.version()).isPresent();
    assertThat(step.version().get()).isEqualTo("2.144");
  }

  @Test
  public void testConstructorWithSingleFormat() {
    RetainNCompositeIndexMigrationStep_2_144 step =
        new RetainNCompositeIndexMigrationStep_2_144(List.of(mavenFormat));

    assertThat(step.version()).isPresent();
    assertThat(step.version().get()).isEqualTo("2.144");
  }
}
