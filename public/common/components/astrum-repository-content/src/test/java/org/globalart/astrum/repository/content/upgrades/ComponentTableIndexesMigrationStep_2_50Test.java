
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
public class ComponentTableIndexesMigrationStep_2_50Test
{
  @Mock
  private Format format;

  private ComponentTableIndexesMigrationStep_2_50 underTest;

  @Before
  public void setup() {
    when(format.getValue()).thenReturn("maven2");
    underTest = new ComponentTableIndexesMigrationStep_2_50(List.of(format));
  }

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();

    assertThat(version).isPresent();
    assertThat(version.get()).isEqualTo("2.50");
  }

  @Test
  public void testConstructorWithEmptyFormats() {
    ComponentTableIndexesMigrationStep_2_50 step =
        new ComponentTableIndexesMigrationStep_2_50(Collections.emptyList());

    assertThat(step.version()).isPresent();
    assertThat(step.version().get()).isEqualTo("2.50");
  }
}
