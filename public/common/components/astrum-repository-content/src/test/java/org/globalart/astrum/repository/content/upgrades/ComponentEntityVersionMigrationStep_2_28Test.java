
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
public class ComponentEntityVersionMigrationStep_2_28Test
{
  @Mock
  private Format format;

  private ComponentEntityVersionMigrationStep_2_28 underTest;

  @Before
  public void setup() {
    when(format.getValue()).thenReturn("maven2");
    underTest = new ComponentEntityVersionMigrationStep_2_28(List.of(format));
  }

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();

    assertThat(version).isPresent();
    assertThat(version.get()).isEqualTo("2.28");
  }

  @Test
  public void testConstructorWithEmptyFormats() {
    ComponentEntityVersionMigrationStep_2_28 step =
        new ComponentEntityVersionMigrationStep_2_28(Collections.emptyList());

    assertThat(step.version()).isPresent();
    assertThat(step.version().get()).isEqualTo("2.28");
  }

  @Test(expected = NullPointerException.class)
  public void testConstructorWithNullFormats() {
    new ComponentEntityVersionMigrationStep_2_28(null);
  }
}
