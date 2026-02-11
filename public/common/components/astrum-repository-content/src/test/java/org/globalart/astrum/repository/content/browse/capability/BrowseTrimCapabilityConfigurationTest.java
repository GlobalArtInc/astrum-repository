
package org.globalart.astrum.repository.content.browse.capability;

import java.util.Map;

import org.junit.Test;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.containsString;
import static org.hamcrest.Matchers.is;

public class BrowseTrimCapabilityConfigurationTest
{
  @Test
  public void testDefaultConstructorHasDefaultValues() {
    BrowseTrimCapabilityConfiguration underTest = new BrowseTrimCapabilityConfiguration();

    assertThat(underTest.isPostgresqlTrimEnabled(), is(false));
    assertThat(underTest.isBatchTrimEnabled(), is(false));
  }

  @Test
  public void testConstructorWithEmptyMap() {
    BrowseTrimCapabilityConfiguration underTest = new BrowseTrimCapabilityConfiguration(Map.of());

    assertThat(underTest.isPostgresqlTrimEnabled(), is(false));
    assertThat(underTest.isBatchTrimEnabled(), is(false));
  }

  @Test
  public void testConstructorWithEnabledProperties() {
    Map<String, String> props = Map.of(
        BrowseTrimCapabilityConfiguration.POSTGRESQL_TRIM_ENABLED, "true",
        BrowseTrimCapabilityConfiguration.BATCH_TRIM_ENABLED, "true");

    BrowseTrimCapabilityConfiguration underTest = new BrowseTrimCapabilityConfiguration(props);

    assertThat(underTest.isPostgresqlTrimEnabled(), is(true));
    assertThat(underTest.isBatchTrimEnabled(), is(true));
  }

  @Test
  public void testConstructorWithDisabledProperties() {
    Map<String, String> props = Map.of(
        BrowseTrimCapabilityConfiguration.POSTGRESQL_TRIM_ENABLED, "false",
        BrowseTrimCapabilityConfiguration.BATCH_TRIM_ENABLED, "false");

    BrowseTrimCapabilityConfiguration underTest = new BrowseTrimCapabilityConfiguration(props);

    assertThat(underTest.isPostgresqlTrimEnabled(), is(false));
    assertThat(underTest.isBatchTrimEnabled(), is(false));
  }

  @Test
  public void testSetters() {
    BrowseTrimCapabilityConfiguration underTest = new BrowseTrimCapabilityConfiguration();

    underTest.setPostgresqlTrimEnabled(true);
    assertThat(underTest.isPostgresqlTrimEnabled(), is(true));

    underTest.setBatchTrimEnabled(true);
    assertThat(underTest.isBatchTrimEnabled(), is(true));
  }

  @Test
  public void testAsMap() {
    BrowseTrimCapabilityConfiguration underTest = new BrowseTrimCapabilityConfiguration();
    underTest.setPostgresqlTrimEnabled(true);
    underTest.setBatchTrimEnabled(false);

    Map<String, String> result = underTest.asMap();
    assertThat(result.get(BrowseTrimCapabilityConfiguration.POSTGRESQL_TRIM_ENABLED), is("true"));
    assertThat(result.get(BrowseTrimCapabilityConfiguration.BATCH_TRIM_ENABLED), is("false"));
  }

  @Test
  public void testToString() {
    BrowseTrimCapabilityConfiguration underTest = new BrowseTrimCapabilityConfiguration();
    String result = underTest.toString();

    assertThat(result, containsString("BrowseTrimCapabilityConfiguration"));
    assertThat(result, containsString("postgresqlTrimEnabled=false"));
    assertThat(result, containsString("batchTrimEnabled=false"));
  }

  @Test
  public void testConstants() {
    assertThat(BrowseTrimCapabilityConfiguration.POSTGRESQL_TRIM_ENABLED, is("postgresqlTrimEnabled"));
    assertThat(BrowseTrimCapabilityConfiguration.BATCH_TRIM_ENABLED, is("batchTrimEnabled"));
  }
}
