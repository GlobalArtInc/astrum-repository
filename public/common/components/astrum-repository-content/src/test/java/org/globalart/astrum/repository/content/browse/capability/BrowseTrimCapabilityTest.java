
package org.globalart.astrum.repository.content.browse.capability;

import java.util.Map;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.instanceOf;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.mockito.Mockito.verify;

@RunWith(MockitoJUnitRunner.Silent.class)
public class BrowseTrimCapabilityTest
{
  @Mock
  private BrowseTrimService browseTrimService;

  private BrowseTrimCapability underTest;

  @Before
  public void setup() {
    underTest = new BrowseTrimCapability(browseTrimService);
  }

  @Test
  public void testCreateConfig() {
    Map<String, String> properties = Map.of(
        BrowseTrimCapabilityConfiguration.POSTGRESQL_TRIM_ENABLED, "true",
        BrowseTrimCapabilityConfiguration.BATCH_TRIM_ENABLED, "false");

    BrowseTrimCapabilityConfiguration config = underTest.createConfig(properties);

    assertThat(config, is(notNullValue()));
    assertThat(config, instanceOf(BrowseTrimCapabilityConfiguration.class));
    assertThat(config.isPostgresqlTrimEnabled(), is(true));
    assertThat(config.isBatchTrimEnabled(), is(false));
  }

  @Test
  public void testOnActivate() throws Exception {
    Map<String, String> properties = Map.of(
        BrowseTrimCapabilityConfiguration.POSTGRESQL_TRIM_ENABLED, "true",
        BrowseTrimCapabilityConfiguration.BATCH_TRIM_ENABLED, "true");
    BrowseTrimCapabilityConfiguration config = new BrowseTrimCapabilityConfiguration(properties);

    underTest.onActivate(config);

    verify(browseTrimService).setPostgresqlTrimEnabled(true);
    verify(browseTrimService).setBatchTrimEnabled(true);
  }

  @Test
  public void testOnActivateDisabled() throws Exception {
    Map<String, String> properties = Map.of(
        BrowseTrimCapabilityConfiguration.POSTGRESQL_TRIM_ENABLED, "false",
        BrowseTrimCapabilityConfiguration.BATCH_TRIM_ENABLED, "false");
    BrowseTrimCapabilityConfiguration config = new BrowseTrimCapabilityConfiguration(properties);

    underTest.onActivate(config);

    verify(browseTrimService).setPostgresqlTrimEnabled(false);
    verify(browseTrimService).setBatchTrimEnabled(false);
  }

  @Test
  public void testOnPassivate() throws Exception {
    BrowseTrimCapabilityConfiguration config = new BrowseTrimCapabilityConfiguration();

    underTest.onPassivate(config);

    verify(browseTrimService).setPostgresqlTrimEnabled(false);
    verify(browseTrimService).setBatchTrimEnabled(false);
  }

  @Test
  public void testOnRemove() throws Exception {
    BrowseTrimCapabilityConfiguration config = new BrowseTrimCapabilityConfiguration();

    underTest.onRemove(config);

    verify(browseTrimService).setPostgresqlTrimEnabled(false);
    verify(browseTrimService).setBatchTrimEnabled(false);
  }

  @Test(expected = NullPointerException.class)
  public void testNullBrowseTrimServiceRejected() {
    new BrowseTrimCapability(null);
  }
}
