package org.globalart.astrum.repository.yum.rest;

import java.util.HashMap;
import java.util.List;
import java.util.Map;

import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.config.internal.ConfigurationData;
import org.globalart.astrum.repository.manager.internal.RepositoryImpl;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;
import org.globalart.astrum.repository.yum.YumFormat;
import org.globalart.astrum.repository.yum.internal.LayoutPolicy;

import org.junit.Before;
import org.junit.Test;

import static org.junit.Assert.assertEquals;
import static org.junit.Assert.assertNull;
import static org.mockito.Mockito.mock;

public class YumApiRepositoryAdapterTest
{
  private YumApiRepositoryAdapter underTest;

  @Before
  public void setUp() {
    underTest = new YumApiRepositoryAdapter(mock(RoutingRuleStore.class));
    BaseUrlHolder.set("http://astrum.example.com", "");
  }

  @Test
  public void adaptsHostedRepositoriesWithTheirYumSettings() throws Exception {
    Repository repository = repository(new HostedType(), Map.of(
        "yum", new HashMap<>(Map.of("repodataDepth", 2, "deployPolicy", "PERMISSIVE")),
        "yumSigning", new HashMap<>(Map.of("keypair", "KEY", "passphrase", "secret"))));

    YumHostedApiRepository adapted = (YumHostedApiRepository) underTest.adapt(repository);

    assertEquals("yum", adapted.getFormat());
    assertEquals("hosted", adapted.getType());
    assertEquals("http://astrum.example.com/repository/yum-repo", adapted.getUrl());
    assertEquals(Integer.valueOf(2), adapted.getYum().getRepodataDepth());
    assertEquals(LayoutPolicy.PERMISSIVE, adapted.getYum().getDeployPolicy());
    assertEquals("KEY", adapted.getYumSigning().getKeypair());
    assertNull(adapted.getYumSigning().getPassphrase());
  }

  @Test
  public void defaultsHostedSettingsThatAreMissing() throws Exception {
    Repository repository = repository(new HostedType(), Map.of());

    YumHostedApiRepository adapted = (YumHostedApiRepository) underTest.adapt(repository);

    assertEquals(Integer.valueOf(0), adapted.getYum().getRepodataDepth());
    assertEquals(LayoutPolicy.STRICT, adapted.getYum().getDeployPolicy());
    assertNull(adapted.getYumSigning());
  }

  @Test
  public void adaptsProxyAndGroupRepositories() throws Exception {
    Repository proxy = repository(new ProxyType(), Map.of(
        "proxy", new HashMap<>(Map.of("remoteUrl", "https://mirror.example.com/el9/", "contentMaxAge", 1440,
            "metadataMaxAge", 60)),
        "yumSigning", new HashMap<>(Map.of("keypair", "KEY"))));
    Repository group = repository(new GroupType(), Map.of(
        "group", new HashMap<>(Map.of("memberNames", List.of("yum-hosted", "yum-proxy")))));

    YumProxyApiRepository adaptedProxy = (YumProxyApiRepository) underTest.adapt(proxy);
    YumGroupApiRepository adaptedGroup = (YumGroupApiRepository) underTest.adapt(group);

    assertEquals("proxy", adaptedProxy.getType());
    assertEquals("https://mirror.example.com/el9/", adaptedProxy.getProxy().getRemoteUrl());
    assertEquals(Integer.valueOf(60), adaptedProxy.getProxy().getMetadataMaxAge());
    assertEquals("KEY", adaptedProxy.getYumSigning().getKeypair());
    assertEquals("group", adaptedGroup.getType());
    assertEquals(List.of("yum-hosted", "yum-proxy"), adaptedGroup.getGroup().getMemberNames());
    assertNull(adaptedGroup.getYumSigning());
  }

  private static Repository repository(final Type type, final Map<String, Map<String, Object>> attributes)
      throws Exception
  {
    ConfigurationData configuration = new ConfigurationData();
    configuration.setRepositoryName("yum-repo");
    configuration.setRecipeName("yum-" + type.getValue());
    configuration.setOnline(true);
    configuration.setAttributes(new HashMap<>(attributes));
    Repository repository = new RepositoryImpl(mock(EventManager.class), type, new YumFormat());
    repository.init(configuration);
    return repository;
  }
}
