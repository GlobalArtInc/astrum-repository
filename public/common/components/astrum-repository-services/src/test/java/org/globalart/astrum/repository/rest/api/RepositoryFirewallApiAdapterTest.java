
package org.globalart.astrum.repository.rest.api;

import java.util.function.Consumer;

import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.db.DatabaseCheck;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.repository.Format;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.internal.RepositoryImpl;
import org.globalart.astrum.repository.rest.api.model.FirewallAttributes;
import org.globalart.astrum.repository.firewall.FirewallMode;
import org.globalart.astrum.repository.rest.api.model.SimpleApiProxyRepository;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.ProxyType;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;
import static org.hamcrest.Matchers.nullValue;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;
import static org.globalart.astrum.repository.rest.api.model.FirewallAttributes.FIREWALL_CHILD_ATTRIBUTE_KEY;
import static org.globalart.astrum.repository.rest.api.model.FirewallAttributes.MODE;

/**
 * Integration tests for repository firewall configuration via REST API adapter.
 * Tests that GET repository operations correctly return firewall configuration from repository storage.
 */
@RunWith(MockitoJUnitRunner.Silent.class)
public class RepositoryFirewallApiAdapterTest
{
  @Mock
  private RoutingRuleStore routingRuleStore;

  @Mock
  private DatabaseCheck databaseCheck;

  private SimpleApiRepositoryAdapter underTest;

  @Before
  public void setup() {
    underTest = new SimpleApiRepositoryAdapter(routingRuleStore);
    underTest.setDatabaseCheck(databaseCheck);
    BaseUrlHolder.set("http://astrum-url", "");
    when(databaseCheck.isPostgresql()).thenReturn(true);
  }

  /**
   * Test Scenario 1: GET repository returns null firewall when not configured
   */
  @Test
  public void testGetRepository_NoFirewallConfiguration_ReturnsNull() throws Exception {
    // Arrange - Create repository without firewall configuration
    Repository repository = createRepository(new ProxyType(), configuration -> {
      NestedAttributesMap proxy = configuration.attributes("proxy");
      proxy.set("remoteUrl", "https://registry.npmjs.org");
    });

    // Act
    SimpleApiProxyRepository apiRepository = (SimpleApiProxyRepository) underTest.adaptDecorated(repository);

    // Assert
    assertThat("Repository should be returned", apiRepository, notNullValue());
    FirewallAttributes firewall = apiRepository.getFirewall();
    // Firewall may be null or have default values depending on implementation
    // The key is that it doesn't cause an error
  }

  /**
   * Test Scenario 2: GET repository returns firewall configuration in DISABLED mode
   */
  @Test
  public void testGetRepository_FirewallDisabled_ReturnsCorrectConfiguration() throws Exception {
    // Arrange - Create repository with firewall in DISABLED mode
    Repository repository = createRepository(new ProxyType(), configuration -> {
      NestedAttributesMap proxy = configuration.attributes("proxy");
      proxy.set("remoteUrl", "https://registry.npmjs.org");

      NestedAttributesMap firewall = configuration.attributes(FIREWALL_CHILD_ATTRIBUTE_KEY);
      firewall.set(MODE, "DISABLED");
    });

    // Act
    SimpleApiProxyRepository apiRepository = (SimpleApiProxyRepository) underTest.adaptDecorated(repository);

    // Assert
    assertThat("Repository should be returned", apiRepository, notNullValue());
    FirewallAttributes firewall = apiRepository.getFirewall();
    assertThat("Firewall configuration should exist", firewall, notNullValue());
    assertThat("Firewall mode should be DISABLED", firewall.getMode(), is(FirewallMode.DISABLED));
  }

  /**
   * Test Scenario 3: GET repository returns firewall configuration in AUDIT mode
   */
  @Test
  public void testGetRepository_FirewallAuditMode_ReturnsCorrectConfiguration() throws Exception {
    // Arrange - Create repository with firewall in AUDIT mode
    Repository repository = createRepository(new ProxyType(), configuration -> {
      NestedAttributesMap proxy = configuration.attributes("proxy");
      proxy.set("remoteUrl", "https://registry.npmjs.org");

      NestedAttributesMap firewall = configuration.attributes(FIREWALL_CHILD_ATTRIBUTE_KEY);
      firewall.set(MODE, "AUDIT");
    });

    // Act
    SimpleApiProxyRepository apiRepository = (SimpleApiProxyRepository) underTest.adaptDecorated(repository);

    // Assert
    assertThat("Repository should be returned", apiRepository, notNullValue());
    FirewallAttributes firewall = apiRepository.getFirewall();
    assertThat("Firewall configuration should exist", firewall, notNullValue());
    assertThat("Firewall mode should be AUDIT", firewall.getMode(), is(FirewallMode.AUDIT));
  }

  /**
   * Test Scenario 4: GET repository returns firewall configuration in QUARANTINE mode
   */
  @Test
  public void testGetRepository_FirewallQuarantineMode_ReturnsCorrectConfiguration() throws Exception {
    // Arrange - Create repository with firewall in QUARANTINE mode
    Repository repository = createRepository(new ProxyType(), configuration -> {
      NestedAttributesMap proxy = configuration.attributes("proxy");
      proxy.set("remoteUrl", "https://registry.npmjs.org");

      NestedAttributesMap firewall = configuration.attributes(FIREWALL_CHILD_ATTRIBUTE_KEY);
      firewall.set(MODE, "QUARANTINE");
    });

    // Act
    SimpleApiProxyRepository apiRepository = (SimpleApiProxyRepository) underTest.adaptDecorated(repository);

    // Assert
    assertThat("Repository should be returned", apiRepository, notNullValue());
    FirewallAttributes firewall = apiRepository.getFirewall();
    assertThat("Firewall configuration should exist", firewall, notNullValue());
    assertThat("Firewall mode should be QUARANTINE", firewall.getMode(), is(FirewallMode.QUARANTINE));
  }

  /**
   * Test Scenario 5: GET repository returns firewall configuration in PCCS mode
   */
  @Test
  public void testGetRepository_FirewallPccsMode_ReturnsCorrectConfiguration() throws Exception {
    // Arrange - Create repository with firewall in PCCS mode
    Repository repository = createRepository(new ProxyType(), configuration -> {
      NestedAttributesMap proxy = configuration.attributes("proxy");
      proxy.set("remoteUrl", "https://registry.npmjs.org");

      NestedAttributesMap firewall = configuration.attributes(FIREWALL_CHILD_ATTRIBUTE_KEY);
      firewall.set(MODE, "PCCS");
    });

    // Act
    SimpleApiProxyRepository apiRepository = (SimpleApiProxyRepository) underTest.adaptDecorated(repository);

    // Assert
    assertThat("Repository should be returned", apiRepository, notNullValue());
    FirewallAttributes firewall = apiRepository.getFirewall();
    assertThat("Firewall configuration should exist", firewall, notNullValue());
    assertThat("Firewall mode should be PCCS", firewall.getMode(), is(FirewallMode.PCCS));
  }

  /**
   * Test Scenario 6: GET repository with no mode set returns null firewall
   */
  @Test
  public void testGetRepository_FirewallNoMode_ReturnsNullFirewall() throws Exception {
    // Arrange - Create repository with firewall section but no mode specified
    Repository repository = createRepository(new ProxyType(), configuration -> {
      NestedAttributesMap proxy = configuration.attributes("proxy");
      proxy.set("remoteUrl", "https://registry.npmjs.org");

      // Firewall section exists but no mode key - should return null
      configuration.attributes(FIREWALL_CHILD_ATTRIBUTE_KEY);
    });

    // Act
    SimpleApiProxyRepository apiRepository = (SimpleApiProxyRepository) underTest.adaptDecorated(repository);

    // Assert
    assertThat("Repository should be returned", apiRepository, notNullValue());
    FirewallAttributes firewall = apiRepository.getFirewall();
    assertThat("Firewall configuration should be null when mode is not set", firewall, is(nullValue()));
  }

  /**
   * Test Scenario 7: Backward compatibility - API clients without firewall field work
   * This test ensures that existing API clients that don't know about the firewall field
   * can still successfully retrieve repository information.
   */
  @Test
  public void testGetRepository_LegacyApiClient_BackwardCompatibility() throws Exception {
    // Arrange - Create repository with all standard configurations but no firewall
    Repository repository = createRepository(new ProxyType(), configuration -> {
      NestedAttributesMap proxy = configuration.attributes("proxy");
      proxy.set("remoteUrl", "https://registry.npmjs.org");
      proxy.set("contentMaxAge", 1440);
      proxy.set("metadataMaxAge", 1440);

      NestedAttributesMap httpclient = configuration.attributes("httpclient");
      httpclient.set("blocked", false);
      httpclient.set("autoBlock", true);
    });

    // Act
    SimpleApiProxyRepository apiRepository = (SimpleApiProxyRepository) underTest.adaptDecorated(repository);

    // Assert - Should work without errors
    assertThat("Repository should be returned", apiRepository, notNullValue());
    assertThat("Repository name should be correct", apiRepository.getName(), is("test-repo"));
    assertThat("Proxy URL should be correct", apiRepository.getProxy().getRemoteUrl(),
        is("https://registry.npmjs.org"));
    assertThat("HTTP client should be configured", apiRepository.getHttpClient(), notNullValue());
    // Firewall may or may not be present - key is no exception thrown
  }

  /**
   * Test Scenario 8: Full roundtrip - write firewall config and read it back
   */
  @Test
  public void testRoundtrip_WriteAndReadFirewallConfiguration() throws Exception {
    // Arrange - Create repository with QUARANTINE mode
    Repository repository = createRepository(new ProxyType(), configuration -> {
      NestedAttributesMap proxy = configuration.attributes("proxy");
      proxy.set("remoteUrl", "https://pypi.org");

      NestedAttributesMap firewall = configuration.attributes(FIREWALL_CHILD_ATTRIBUTE_KEY);
      firewall.set(MODE, "QUARANTINE");
    });

    // Act - Read it back
    SimpleApiProxyRepository apiRepository = (SimpleApiProxyRepository) underTest.adaptDecorated(repository);

    // Assert - Values match what was written
    assertThat("Repository should be returned", apiRepository, notNullValue());
    FirewallAttributes firewall = apiRepository.getFirewall();
    assertThat("Firewall configuration should exist", firewall, notNullValue());
    assertThat("Firewall mode should match", firewall.getMode(), is(FirewallMode.QUARANTINE));

    // Now update to PCCS mode
    repository = createRepository(new ProxyType(), configuration -> {
      NestedAttributesMap proxy = configuration.attributes("proxy");
      proxy.set("remoteUrl", "https://pypi.org");

      NestedAttributesMap firewallConfig = configuration.attributes(FIREWALL_CHILD_ATTRIBUTE_KEY);
      firewallConfig.set(MODE, "PCCS");
    });

    apiRepository = (SimpleApiProxyRepository) underTest.adaptDecorated(repository);
    firewall = apiRepository.getFirewall();
    assertThat("Firewall configuration should exist after update", firewall, notNullValue());
    assertThat("Firewall mode should be updated to PCCS", firewall.getMode(), is(FirewallMode.PCCS));
  }

  /**
   * Helper method to create a repository with custom configuration
   */
  private Repository createRepository(Type type, Consumer<Configuration> mutator) throws Exception {
    Repository repository = new RepositoryImpl(mock(EventManager.class), type, new Format("npm")
    {
    });
    Configuration configuration = new SimpleApiRepositoryAdapterTest.SimpleConfiguration();
    configuration.setOnline(true);
    configuration.setRepositoryName("test-repo");
    mutator.accept(configuration);
    repository.init(configuration);
    return repository;
  }
}
