
package org.globalart.astrum.content.maven.internal.recipe;

import java.io.IOException;
import java.net.URI;
import java.util.Objects;
import javax.annotation.Nonnull;
import javax.annotation.Nullable;
import org.springframework.beans.factory.annotation.Autowired;
import jakarta.validation.ConstraintViolation;
import jakarta.validation.ConstraintViolationException;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.content.maven.MavenContentFacet;
import org.globalart.astrum.repository.cache.CacheController;
import org.globalart.astrum.repository.cache.CacheInfo;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.facet.ContentProxyFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.maven.LayoutPolicy;
import org.globalart.astrum.repository.maven.MavenPath;
import org.globalart.astrum.repository.maven.MavenProxyRequestHeaderSupport;
import org.globalart.astrum.repository.maven.internal.Constants;
import org.globalart.astrum.repository.proxy.ProxyFacetSupport;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import com.google.common.collect.ImmutableSet;
import com.google.common.net.HttpHeaders;
import org.apache.http.client.methods.HttpRequestBase;

import static com.google.common.base.Preconditions.checkNotNull;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

/**
 * Maven specific implementation of {@link ProxyFacetSupport}.
 *
 * @since 3.26
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class MavenProxyFacet
    extends ContentProxyFacetSupport
{
  private static final String MAVEN_CENTRAL_HOST = "repo1.maven.org";

  public static final String P_ATTRIBUTES = "attributes";

  final ConstraintViolationFactory constraintViolationFactory;

  private final MavenProxyRequestHeaderSupport mavenProxyRequestHeaderSupport;

  @Autowired
  public MavenProxyFacet(
      final ConstraintViolationFactory constraintViolationFactory,
      final MavenProxyRequestHeaderSupport mavenProxyRequestHeaderSupport)
  {
    this.constraintViolationFactory = checkNotNull(constraintViolationFactory);
    this.mavenProxyRequestHeaderSupport = checkNotNull(mavenProxyRequestHeaderSupport);
  }

  @Override
  protected void doValidate(final Configuration configuration) throws Exception {
    super.doValidate(configuration);
    validatePreemptiveAuth(configuration);
  }

  /**
   * Pre-emptive authentication is limited only to Maven Proxy using HTTPS protocol.
   */
  private void validatePreemptiveAuth(final Configuration configuration) {
    NestedAttributesMap httpclient = configuration.attributes("httpclient");
    if (Objects.nonNull(httpclient.get("authentication"))) {
      boolean isPreemptive =
          configuration.attributes("httpclient").child("authentication").get("preemptive", Boolean.class, false);
      if (isPreemptive) {
        String remoteUrl = configuration.attributes("proxy").get("remoteUrl", String.class, "");
        if (!remoteUrl.startsWith("https://")) {
          ConstraintViolation<?> violation = constraintViolationFactory
              .createViolation(P_ATTRIBUTES + ".httpclient.authentication.preemptive",
                  "Pre-emptive authentication is allowed only when using HTTPS remote URL");
          throw new ConstraintViolationException(ImmutableSet.of(violation));
        }
      }
    }
  }

  @Override
  protected Content getCachedContent(final Context context) throws IOException {
    return content()
        .get(mavenPath(context))
        .orElse(null);
  }

  @Nullable
  @Override
  protected Content fetch(final Context context, final Content stale) throws IOException {
    // do not go remote for a non maven 2 artifact or metadata if is not already present in cache or allowed by config
    MavenContentFacet mavenFacet = content();
    if (stale == null && mavenFacet.layoutPolicy() == LayoutPolicy.STRICT) {
      MavenPath mavenPath = mavenPath(context);
      if (mavenPath.getCoordinates() == null
          && !mavenFacet.getMavenPathParser().isRepositoryMetadata(mavenPath)
          && !mavenFacet.getMavenPathParser().isRepositoryIndex(mavenPath)
          && !mavenPath.getFileName().equals(Constants.ARCHETYPE_CATALOG_FILENAME)) {
        return null;
      }
    }
    return super.fetch(context, stale);
  }

  @Override
  protected CacheController getCacheController(@Nonnull final Context context) {
    if (content().getMavenPathParser().isRepositoryMetadata(mavenPath(context))) {
      return cacheControllerHolder.getMetadataCacheController();
    }
    else {
      return cacheControllerHolder.getContentCacheController();
    }
  }

  @Override
  protected Content store(final Context context, final Content payload) throws IOException {
    return content().put(mavenPath(context), payload);
  }

  @Override
  protected String getUrl(@Nonnull final Context context) {
    return removePrefixingSlash(context.getRequest().getPath());
  }

  // NOTE: isNotModified() with ETag-based blob reuse is now handled by ContentProxyFacetSupport
  // The logic was moved there to benefit ALL content-based proxy formats automatically.

  @Override
  protected void indicateVerified(
      final Context context,
      final Content content,
      final CacheInfo cacheInfo)
  {
    MavenPath mavenPath = mavenPath(context);
    FluentAsset fluentAsset = facet(MavenContentFacet.class)
        .createComponentAndAsset(mavenPath);

    if (fluentAsset == null) {
      log.debug("Attempting to set cache info for non-existent maven asset: {} on the repository: {}",
          mavenPath, getRepository().getName());
      return;
    }

    if (log.isDebugEnabled()) {
      log.debug("Updating cacheInfo of {} to {} on the repository {}",
          fluentAsset.path(), cacheInfo, getRepository().getName());
    }
    fluentAsset.markAsCached(cacheInfo);
  }

  @Override
  protected HttpRequestBase buildFetchHttpRequest(final URI uri, final Context context) {
    HttpRequestBase request = super.buildFetchHttpRequest(uri, context);
    String augmentedUserAgent;
    if (MAVEN_CENTRAL_HOST.equals(uri.getHost())) {
      augmentedUserAgent = mavenProxyRequestHeaderSupport.getUserAgentForAnalytics();
      request.setHeader(HttpHeaders.USER_AGENT, augmentedUserAgent);
    }
    return request;
  }

  @Nonnull
  private MavenPath mavenPath(@Nonnull final Context context) {
    return context.getAttributes().require(MavenPath.class);
  }

  private MavenContentFacet content() {
    return getRepository().facet(MavenContentFacet.class);
  }

  private String removePrefixingSlash(final String url) {
    if (url != null && url.startsWith("/")) {
      return url.replaceFirst("/", "");
    }
    return url;
  }
}
