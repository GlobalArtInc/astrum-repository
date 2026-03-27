
package org.globalart.astrum.repository.maven.rest;

import java.util.Arrays;

import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.config.ConfigurationStore;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.rest.api.model.GroupAttributes;
import org.globalart.astrum.repository.rest.api.model.StorageAttributes;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.Mockito.when;

/**
 * Tests for {@link MavenGroupRepositoryApiRequestToConfigurationConverter}.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class MavenGroupRepositoryApiRequestToConfigurationConverterTest

{
  @Mock
  private ConfigurationStore configurationStore;

  private MavenGroupRepositoryApiRequestToConfigurationConverter underTest;

  @Before
  public void setUp() {
    when(configurationStore.newConfiguration()).thenReturn(new ConfigurationData());
    underTest = new MavenGroupRepositoryApiRequestToConfigurationConverter();
    underTest.setConfigurationStore(configurationStore);
  }

  @Test
  public void testConvert_groupAttributesSet() {
    MavenGroupRepositoryApiRequest request = createRequest();
    Configuration config = underTest.convert(request);

    assertThat(config, notNullValue());
    assertThat(config.attributes("group").get("memberNames"), notNullValue());
  }

  private MavenGroupRepositoryApiRequest createRequest() {
    StorageAttributes storage = new StorageAttributes("default", true);
    GroupAttributes group = new GroupAttributes(Arrays.asList("maven-hosted", "maven-central"));

    return new MavenGroupRepositoryApiRequest(
        "maven-group",
        true,
        storage,
        group);
  }
}
