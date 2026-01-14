
package org.globalart.astrum.rest.client.internal;

import java.net.URI;

import org.springframework.beans.factory.annotation.Autowired;
import jakarta.inject.Provider;
import jakarta.ws.rs.client.Client;
import jakarta.ws.rs.client.WebTarget;

import org.globalart.astrum.common.thread.TcclBlock;
import org.globalart.astrum.httpclient.SSLContextSelector;
import org.globalart.astrum.rest.client.RestClientConfiguration;
import org.globalart.astrum.rest.client.RestClientFactory;

import com.google.common.cache.CacheBuilder;
import com.google.common.cache.LoadingCache;
import org.apache.http.client.HttpClient;
import org.apache.http.protocol.BasicHttpContext;
import org.apache.http.protocol.HttpContext;
import org.jboss.resteasy.client.jaxrs.ClientHttpEngine;
import org.jboss.resteasy.client.jaxrs.ProxyBuilder;
import org.jboss.resteasy.client.jaxrs.ResteasyClientBuilder;
// ASTRUM-46395: ApacheHttpClient4Engine (Apache HttpClient 4.x) was removed in RESTEasy 7.
// We migrated to ApacheHttpClient43Engine (Apache HttpClient 4.3+) since the application-
// wide HttpClient is still HttpClient 4.x; a future bump to Apache HttpClient 5 +
// ApacheHttpClient5Engine is tracked separately.
import org.jboss.resteasy.client.jaxrs.engines.ApacheHttpClient43Engine;
import org.jboss.resteasy.client.jaxrs.engines.HttpContextProvider;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.context.annotation.Primary;
import org.springframework.stereotype.Component;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static com.google.common.base.Preconditions.checkNotNull;
import static com.google.common.cache.CacheLoader.from;

/**
 * REST client factory.
 *
 * @since 3.0
 */
@Primary
@Component
@Qualifier("default")
public class RestClientFactoryImpl
    implements RestClientFactory
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final LoadingCache<ClassLoader, ClassLoader> bridgeClassLoaderCache =
      CacheBuilder.newBuilder()
          .build(from(
              (loader) -> new BridgeClassLoader(loader, ProxyBuilder.class.getClassLoader())));

  private final Provider<HttpClient> httpClient;

  @Autowired
  public RestClientFactoryImpl(final Provider<HttpClient> httpClient) {
    this.httpClient = checkNotNull(httpClient);
  }

  @Override
  public Client create(final RestClientConfiguration configuration) {
    checkNotNull(configuration);

    try (TcclBlock tccl = TcclBlock.begin(ResteasyClientBuilder.class)) {
      final HttpClient client;
      if (configuration.getHttpClient() != null) {
        client = checkNotNull(configuration.getHttpClient().get());
      }
      else {
        client = httpClient.get();
      }

      // ASTRUM-46395: build a fresh BasicHttpContext per request and route
      // SSLContextSelector.USE_TRUST_STORE through it when the caller has asked for the
      // Astrum-managed truststore. RESTEasy 7's ApacheHttpClient43Engine consumes the
      // context via HttpContextProvider#getContext (replacing v3's plain (HttpClient,
      // HttpContext) ctor); we hand it a fresh context per call so concurrent invocations
      // don't share mutable state.
      final boolean useTrustStore = configuration.getUseTrustStore();
      HttpContextProvider contextProvider = () -> {
        HttpContext ctx = new BasicHttpContext();
        if (useTrustStore) {
          ctx.setAttribute(SSLContextSelector.USE_TRUST_STORE, true);
        }
        return ctx;
      };

      // ASTRUM-46395: the (HttpClient, HttpContextProvider) ctor sets
      // closeHttpClient=false internally. The (HttpClient, boolean) ctor defaults that
      // flag to true, which would dispose the application-wide pooled HttpClient on the
      // first Client#close() and break every subsequent caller (replication, IQ, S3
      // metadata, etc.). Always go through the provider form.
      ClientHttpEngine httpEngine = new ApacheHttpClient43Engine(client, contextProvider);

      // ASTRUM-46395: ResteasyClientBuilder became abstract in RESTEasy 7; obtain an
      // implementation through the JAX-RS ClientBuilder SPI and cast.
      ResteasyClientBuilder builder =
          (ResteasyClientBuilder) jakarta.ws.rs.client.ClientBuilder.newBuilder();
      builder.httpEngine(httpEngine);

      if (configuration.getCustomizer() != null) {
        configuration.getCustomizer().apply(builder);
      }

      return builder.build();
    }
  }

  @Override
  public <T> T proxy(final Class<T> api, final Client client, final URI baseUri) {
    WebTarget target = client.target(baseUri);

    return ProxyBuilder.builder(api, target)
        .classloader(bridgeClassLoaderCache.getUnchecked(api.getClassLoader()))
        .build();
  }
}
