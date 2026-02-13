
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
public class AssetBlobExternalMetadataMigrationStep_2_70Test
{
  @Mock
  private Format format;

  private AssetBlobExternalMetadataMigrationStep_2_70 underTest;

  @Before
  public void setup() {
    when(format.getValue()).thenReturn("maven2");
    underTest = new AssetBlobExternalMetadataMigrationStep_2_70(List.of(format));
  }

  @Test
  public void testVersion() {
    Optional<String> version = underTest.version();

    assertThat(version).isPresent();
    assertThat(version.get()).isEqualTo("2.70");
  }

  @Test
  public void testConstructorWithEmptyFormats() {
    AssetBlobExternalMetadataMigrationStep_2_70 step =
        new AssetBlobExternalMetadataMigrationStep_2_70(Collections.emptyList());

    assertThat(step.version()).isPresent();
    assertThat(step.version().get()).isEqualTo("2.70");
  }

  @Test(expected = NullPointerException.class)
  public void testConstructorWithNullFormats() {
    new AssetBlobExternalMetadataMigrationStep_2_70(null);
  }
}
