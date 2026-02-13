
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
public class KeyValueIndexesMigrationStep_2_65Test
{
  @Mock
  private Format format;

  private KeyValueIndexesMigrationStep_2_65 underTest;

  @Before
  public void setup() {
    when(format.getValue()).thenReturn("maven2");
    underTest = new KeyValueIndexesMigrationStep_2_65(List.of(format));
  }

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();

    assertThat(version).isPresent();
    assertThat(version.get()).isEqualTo("2.65");
  }

  @Test
  public void testConstructorWithEmptyFormats() {
    KeyValueIndexesMigrationStep_2_65 step =
        new KeyValueIndexesMigrationStep_2_65(Collections.emptyList());

    assertThat(step.version()).isPresent();
    assertThat(step.version().get()).isEqualTo("2.65");
  }
}
