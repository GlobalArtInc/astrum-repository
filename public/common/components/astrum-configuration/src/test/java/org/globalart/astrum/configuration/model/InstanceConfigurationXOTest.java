
package org.globalart.astrum.configuration.model;

import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;

import org.junit.Before;
import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.contains;
import static org.hamcrest.Matchers.empty;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.hasSize;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.not;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.Matchers.sameInstance;

public class InstanceConfigurationXOTest
{
  private InstanceConfigurationXO underTest;

  private ConfigurationXO config1;

  private ConfigurationXO config2;

  private ConfigurationXO config3;

  @Before
  public void setUp() {
    underTest = new InstanceConfigurationXO();
    config1 = createMockConfig("type1");
    config2 = createMockConfig("type2");
    config3 = createMockConfig("type3");
  }

  @Test
  public void testInitialState() {
    assertThat(underTest.getConfigurationXOs(), notNullValue());
    assertThat(underTest.getConfigurationXOs(), empty());
  }

  @Test
  public void testAddConfigurationXO_withValidConfig() {
    underTest.addConfigurationXO(config1);

    assertThat(underTest.getConfigurationXOs(), hasSize(1));
    assertThat(underTest.getConfigurationXOs(), contains(config1));
  }

  @Test
  public void testAddConfigurationXO_withNull() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(null);
    underTest.addConfigurationXO(config2);

    assertThat(underTest.getConfigurationXOs(), hasSize(2));
    assertThat(underTest.getConfigurationXOs(), contains(config1, config2));
  }

  @Test
  public void testAddConfigurationXO_multipleConfigs() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);
    underTest.addConfigurationXO(config3);

    assertThat(underTest.getConfigurationXOs(), hasSize(3));
    assertThat(underTest.getConfigurationXOs(), contains(config1, config2, config3));
  }

  @Test
  public void testSetConfigurationXOs_withValidList() {
    List<ConfigurationXO> configs = Arrays.asList(config1, config2, config3);
    underTest.setConfigurationXOs(configs);

    assertThat(underTest.getConfigurationXOs(), hasSize(3));
    assertThat(underTest.getConfigurationXOs(), contains(config1, config2, config3));
  }

  @Test
  public void testSetConfigurationXOs_filtersNullValues() {
    List<ConfigurationXO> configs = Arrays.asList(config1, null, config2, null, config3);
    underTest.setConfigurationXOs(configs);

    assertThat(underTest.getConfigurationXOs(), hasSize(3));
    assertThat(underTest.getConfigurationXOs(), contains(config1, config2, config3));
  }

  @Test
  public void testSetConfigurationXOs_withEmptyList() {
    underTest.addConfigurationXO(config1);
    underTest.setConfigurationXOs(new ArrayList<>());

    assertThat(underTest.getConfigurationXOs(), empty());
  }

  @Test
  public void testSetConfigurationXOs_withAllNullValues() {
    List<ConfigurationXO> configs = Arrays.asList(null, null, null);
    underTest.setConfigurationXOs(configs);

    assertThat(underTest.getConfigurationXOs(), empty());
  }

  @Test
  public void testSetConfigurationXOs_replacesExistingList() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);

    List<ConfigurationXO> newConfigs = Arrays.asList(config3);
    underTest.setConfigurationXOs(newConfigs);

    assertThat(underTest.getConfigurationXOs(), hasSize(1));
    assertThat(underTest.getConfigurationXOs(), contains(config3));
  }

  @Test
  public void testGetConfigurationXO_findsMatchingConfig() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);
    underTest.addConfigurationXO(config3);

    ConfigurationXO result = underTest.getConfigurationXO("type2");

    assertThat(result, notNullValue());
    assertThat(result, sameInstance(config2));
  }

  @Test
  public void testGetConfigurationXO_returnsNullWhenNotFound() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);

    ConfigurationXO result = underTest.getConfigurationXO("nonexistent");

    assertThat(result, nullValue());
  }

  @Test
  public void testGetConfigurationXO_returnsFirstMatchWhenDuplicates() {
    ConfigurationXO duplicateConfig = createMockConfig("type1");
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(duplicateConfig);

    ConfigurationXO result = underTest.getConfigurationXO("type1");

    assertThat(result, sameInstance(config1));
  }

  @Test
  public void testGetConfigurationXO_withEmptyList() {
    ConfigurationXO result = underTest.getConfigurationXO("type1");

    assertThat(result, nullValue());
  }

  @Test
  public void testGetConfigurationXO_doesNotThrowNPEWithNullsInList() {
    // This test verifies the fix for the NPE issue when null values are in the list
    List<ConfigurationXO> configs = Arrays.asList(config1, null, config2);
    underTest.setConfigurationXOs(configs);

    // Should not throw NPE because setConfigurationXOs filters nulls
    ConfigurationXO result = underTest.getConfigurationXO("type2");

    assertThat(result, sameInstance(config2));
  }

  @Test
  public void testEquals_sameInstance() {
    assertThat(underTest.equals(underTest), is(true));
  }

  @Test
  public void testEquals_nullObject() {
    assertThat(underTest.equals(null), is(false));
  }

  @Test
  public void testEquals_differentClass() {
    assertThat(underTest.equals("not an InstanceConfigurationXO"), is(false));
  }

  @Test
  public void testEquals_emptyLists() {
    InstanceConfigurationXO other = new InstanceConfigurationXO();

    assertThat(underTest.equals(other), is(true));
  }

  @Test
  public void testEquals_sameLists() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);

    InstanceConfigurationXO other = new InstanceConfigurationXO();
    other.addConfigurationXO(config1);
    other.addConfigurationXO(config2);

    assertThat(underTest.equals(other), is(true));
  }

  @Test
  public void testEquals_differentLists() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);

    InstanceConfigurationXO other = new InstanceConfigurationXO();
    other.addConfigurationXO(config1);
    other.addConfigurationXO(config3);

    assertThat(underTest.equals(other), is(false));
  }

  @Test
  public void testEquals_differentSizeLists() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);

    InstanceConfigurationXO other = new InstanceConfigurationXO();
    other.addConfigurationXO(config1);

    assertThat(underTest.equals(other), is(false));
  }

  @Test
  public void testHashCode_emptyList() {
    InstanceConfigurationXO other = new InstanceConfigurationXO();

    assertThat(underTest.hashCode(), equalTo(other.hashCode()));
  }

  @Test
  public void testHashCode_sameLists() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);

    InstanceConfigurationXO other = new InstanceConfigurationXO();
    other.addConfigurationXO(config1);
    other.addConfigurationXO(config2);

    assertThat(underTest.hashCode(), equalTo(other.hashCode()));
  }

  @Test
  public void testHashCode_differentLists() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);

    InstanceConfigurationXO other = new InstanceConfigurationXO();
    other.addConfigurationXO(config3);

    assertThat(underTest.hashCode(), not(equalTo(other.hashCode())));
  }

  @Test
  public void testHashCode_consistency() {
    underTest.addConfigurationXO(config1);
    int hash1 = underTest.hashCode();
    int hash2 = underTest.hashCode();

    assertThat(hash1, equalTo(hash2));
  }

  @Test
  public void testGetConfigurationXOs_returnsInternalList() {
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);

    List<ConfigurationXO> result = underTest.getConfigurationXOs();

    assertThat(result, hasSize(2));
    assertThat(result, contains(config1, config2));
  }

  @Test
  public void testComplexScenario_addThenSetThenGet() {
    // Add some configs
    underTest.addConfigurationXO(config1);
    underTest.addConfigurationXO(config2);

    // Replace with new list (with nulls that should be filtered)
    List<ConfigurationXO> newConfigs = Arrays.asList(config2, null, config3);
    underTest.setConfigurationXOs(newConfigs);

    // Verify the result
    assertThat(underTest.getConfigurationXOs(), hasSize(2));
    assertThat(underTest.getConfigurationXO("type2"), sameInstance(config2));
    assertThat(underTest.getConfigurationXO("type3"), sameInstance(config3));
    assertThat(underTest.getConfigurationXO("type1"), nullValue());
  }

  /**
   * Helper method to create a mock ConfigurationXO with a specific type ID
   */
  private ConfigurationXO createMockConfig(final String typeId) {
    return new ConfigurationXO()
    {
      @Override
      public String getConfigurationTypeId() {
        return typeId;
      }

      @Override
      public String toString() {
        return "MockConfig[" + typeId + "]";
      }
    };
  }
}
