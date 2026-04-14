
package org.globalart.astrum.repository.apt.api;

import java.util.Map;

import org.globalart.astrum.common.app.BaseUrlHolder;
import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.event.EventManager;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.manager.internal.RepositoryImpl;
import org.globalart.astrum.repository.rest.api.model.AbstractApiRepository;
import org.globalart.astrum.repository.routing.RoutingRuleStore;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.repository.types.ProxyType;

import org.junit.Before;
import org.junit.Test;
import org.junit.runner.RunWith;
import org.mockito.Mock;
import org.mockito.junit.MockitoJUnitRunner;

import static com.google.common.collect.Maps.newHashMap;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.nullValue;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.mockito.AdditionalMatchers.not;
import static org.mockito.ArgumentMatchers.startsWith;
import static org.mockito.Mockito.mock;
import static org.mockito.Mockito.when;

@RunWith(MockitoJUnitRunner.class)
public class AptApiRepositoryAdapterTest
{
  private AptApiRepositoryAdapter underTest;

  @Mock
  private RoutingRuleStore routingRuleStore;

  @Before
  public void setup() {
    underTest = new AptApiRepositoryAdapter(routingRuleStore);
    BaseUrlHolder.set("http://astrum-url", "");
  }

  @Test
  public void testAdapt_hostedRepository() throws Exception {
    Repository repository = createRepository(new HostedType(), "bionic", "asdf", null, null);

    AptHostedApiRepository hostedRepository = (AptHostedApiRepository) underTest.adapt(repository);
    assertRepository(hostedRepository, "hosted", true);
    assertThat(hostedRepository.getApt().getDistribution(), is("bionic"));
    assertThat(hostedRepository.getAptSigning(), org.hamcrest.Matchers.not(nullValue()));
    assertThat(hostedRepository.getAptSigning().getKeypair(), is("asdf"));
    assertThat(hostedRepository.getAptSigning().getPassphrase(), nullValue());

    repository.getConfiguration().attributes("aptSigning").set("passphrase", "mypass");
    hostedRepository = (AptHostedApiRepository) underTest.adapt(repository);
    assertThat(hostedRepository.getAptSigning().getKeypair(), is("asdf"));
    assertThat(hostedRepository.getAptSigning().getPassphrase(), nullValue());

    repository.getConfiguration().attributes("aptSigning").set("keypair", "");
    hostedRepository = (AptHostedApiRepository) underTest.adapt(repository);
    assertThat(hostedRepository.getAptSigning(), nullValue());
  }

  @Test
  public void testAdapt_hostedRepository_passphraseOnly() throws Exception {
    Repository repository = createRepository(new HostedType(), "bionic", null, "mypass", null);

    AptHostedApiRepository hostedRepository = (AptHostedApiRepository) underTest.adapt(repository);
    // aptSigning should be null because keypair is not set
    assertThat(hostedRepository.getAptSigning(), nullValue());
  }

  @Test
  public void testAdapt_proxyRepository() throws Exception {
    Repository repository = createRepository(new ProxyType(), "bionic", null, null, true);

    AptProxyApiRepository proxyRepository = (AptProxyApiRepository) underTest.adapt(repository);
    assertRepository(proxyRepository, "proxy", true);
    assertThat(proxyRepository.getApt().getDistribution(), is("bionic"));
    assertThat(proxyRepository.getApt().getFlat(), is(true));
    assertThat(proxyRepository.getAptSigning(), nullValue());
  }

  @Test
  public void testAdapt_proxyRepository_withKeypair() throws Exception {
    Repository repository = createRepository(new ProxyType(), "bionic", "proxy-keypair", null, true);

    AptProxyApiRepository proxyRepository = (AptProxyApiRepository) underTest.adapt(repository);
    assertRepository(proxyRepository, "proxy", true);
    assertThat(proxyRepository.getApt().getDistribution(), is("bionic"));
    assertThat(proxyRepository.getAptSigning(), org.hamcrest.Matchers.not(nullValue()));
    assertThat(proxyRepository.getAptSigning().getKeypair(), is("proxy-keypair"));
    assertThat(proxyRepository.getAptSigning().getPassphrase(), nullValue());

    repository.getConfiguration().attributes("aptSigning").set("passphrase", "pass");
    assertThat(proxyRepository.getAptSigning().getKeypair(), is("proxy-keypair"));
    assertThat(proxyRepository.getAptSigning().getPassphrase(), nullValue());

    repository.getConfiguration().attributes("aptSigning").set("keypair", "");
    proxyRepository = (AptProxyApiRepository) underTest.adapt(repository);
    assertThat(proxyRepository.getAptSigning(), nullValue());
  }

  private static void assertRepository(
      final AbstractApiRepository repository,
      final String type,
      final Boolean online)
  {
    assertThat(repository.getFormat(), is("apt"));
    assertThat(repository.getName(), is("my-repo"));
    assertThat(repository.getOnline(), is(online));
    assertThat(repository.getType(), is(type));
    assertThat(repository.getUrl(), is(BaseUrlHolder.get() + "/repository/my-repo"));
  }

  private static Configuration config(final String repositoryName) {
    Configuration configuration = mock(Configuration.class);
    when(configuration.isOnline()).thenReturn(true);
    when(configuration.getRepositoryName()).thenReturn(repositoryName);
    when(configuration.attributes(not(startsWith("apt"))))
        .thenReturn(new NestedAttributesMap("dummy", newHashMap()));
    return configuration;
  }

  private static Repository createRepository(
      final Type type,
      final String distribution,
      final String keypair,
      final String passphrase,
      final Boolean flat) throws Exception
  {
    Repository repository = new RepositoryImpl(mock(EventManager.class), type, new AptFormat());

    Configuration configuration = config("my-repo");
    Map<String, Object> attributes = newHashMap();

    Map<String, Object> apt = newHashMap();
    apt.put("distribution", distribution);
    apt.put("flat", flat);
    attributes.put("apt", apt);
    NestedAttributesMap aptNested = new NestedAttributesMap("apt", apt);
    when(configuration.attributes("apt")).thenReturn(aptNested);

    Map<String, Object> aptSigning = newHashMap();
    aptSigning.put("keypair", keypair);
    aptSigning.put("passphrase", passphrase);
    attributes.put("aptSigning", aptSigning);
    NestedAttributesMap aptSigningNested = new NestedAttributesMap("aptSigning", aptSigning);
    when(configuration.attributes("aptSigning")).thenReturn(aptSigningNested);

    repository.init(configuration);
    return repository;
  }
}
