
package org.globalart.astrum.repository.conan.internal.proxy;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.Map;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.conan.internal.ConanEndpoint;
import org.globalart.astrum.repository.conan.internal.ConanJson;
import org.globalart.astrum.repository.conan.internal.hosted.ConanHostedHandlers;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.httpclient.HttpClientFacet;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import org.apache.http.HttpResponse;
import org.apache.http.client.config.RequestConfig;
import org.apache.http.client.methods.HttpGet;
import org.apache.http.client.utils.URIBuilder;
import org.apache.http.util.EntityUtils;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

@Component
public class ConanProxyHandlers
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final int searchTimeout;

  public ConanProxyHandlers(@Value("${astrum.conan.proxy.search.timeout:120000}") final int searchTimeout) {
    this.searchTimeout = searchTimeout;
  }

  public final Handler protocol = context -> {
    ConanEndpoint endpoint = ConanEndpoint.of(context);
    if (!context.getRepository().facet(ConanProxyFacet.class).supports(endpoint)) {
      return HttpResponses.notFound();
    }
    return context.proceed();
  };

  public final Handler search = context -> {
    Repository repository = context.getRepository();
    try {
      Map<String, Object> results = remoteSearch(context, repository);
      if (results != null) {
        return ConanJson.ok(results);
      }
    }
    catch (IOException | URISyntaxException | RuntimeException e) {
      log.warn("Remote Conan search failed for repository {}: {}", repository.getName(), e.toString());
    }
    return ConanHostedHandlers.search(context);
  };

  private Map<String, Object> remoteSearch(final Context context, final Repository repository)
      throws IOException, URISyntaxException
  {
    URI remote = repository.facet(ConanProxyFacet.class).getRemoteUrl();
    URIBuilder builder = new URIBuilder(remote.resolve(context.getRequest().getPath().substring(1)));
    for (Map.Entry<String, String> parameter : context.getRequest().getParameters()) {
      builder.addParameter(parameter.getKey(), parameter.getValue());
    }
    HttpGet get = new HttpGet(builder.build());
    get.setConfig(RequestConfig.custom().setSocketTimeout(searchTimeout).build());
    HttpResponse response = repository.facet(HttpClientFacet.class).getHttpClient().execute(get);
    try {
      if (response.getStatusLine().getStatusCode() != 200 || response.getEntity() == null) {
        log.debug("Remote Conan search of {} returned {}", repository.getName(), response.getStatusLine());
        return null;
      }
      return ConanJson.read(EntityUtils.toByteArray(response.getEntity()));
    }
    finally {
      EntityUtils.consumeQuietly(response.getEntity());
    }
  }
}
