
package org.globalart.astrum.repository.pypi.internal.content;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.time.Duration;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Optional;
import java.util.function.Supplier;

import javax.annotation.Nullable;

import org.globalart.astrum.common.cooperation2.Cooperation2;
import org.globalart.astrum.common.cooperation2.Cooperation2Factory;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.repository.Facet.Exposed;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.RepositoryCacheInvalidationService;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.event.asset.AssetCreatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetDeletedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetEvent;
import org.globalart.astrum.repository.content.event.asset.AssetUploadedEvent;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.group.GroupFacetImpl;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.pypi.content.PypiContentFacet;
import org.globalart.astrum.repository.pypi.internal.AssetKind;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.StringPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.validation.ConstraintViolationFactory;

import com.google.common.eventbus.AllowConcurrentEvents;
import com.google.common.eventbus.Subscribe;
import org.joda.time.DateTime;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static java.util.Objects.isNull;
import static org.globalart.astrum.repository.pypi.internal.AssetKind.INDEX;
import static org.globalart.astrum.repository.pypi.internal.AssetKind.ROOT_INDEX;
import static org.globalart.astrum.repository.pypi.internal.PyPiStorageUtils.HASH_ALGORITHMS;
import static org.globalart.astrum.repository.view.Content.CONTENT_LAST_MODIFIED;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_HTML;

@Exposed
@Component
@Qualifier(PyPiFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class PyPiGroupFacet
    extends GroupFacetImpl
    implements EventAware.Asynchronous
{
  @Nullable
  private Cooperation2Factory.Builder cooperationBuilder;

  @Nullable
  private Cooperation2 indexCooperation;

  @Autowired
  public PyPiGroupFacet(
      final RepositoryManager repositoryManager,
      final ConstraintViolationFactory constraintViolationFactory,
      @Qualifier(GroupType.NAME) final Type groupType,
      final RepositoryCacheInvalidationService repositoryCacheInvalidationService)
  {
    super(repositoryManager, constraintViolationFactory, groupType, repositoryCacheInvalidationService);
  }

  @Autowired
  protected void configureCooperation(
      final Cooperation2Factory cooperationFactory,
      @Value("${astrum.pypi.indexRoot.cooperation.enabled:true}") final boolean cooperationEnabled,
      @Value("${astrum.pypi.indexRoot.cooperation.majorTimeout:0s}") final Duration majorTimeout,
      @Value("${astrum.pypi.indexRoot.cooperation.minorTimeout:30s}") final Duration minorTimeout,
      @Value("${astrum.pypi.indexRoot.cooperation.threadsPerKey:100}") final int threadsPerKey)
  {
    this.cooperationBuilder = cooperationFactory.configure()
        .enabled(cooperationEnabled)
        .majorTimeout(majorTimeout)
        .minorTimeout(minorTimeout)
        .threadsPerKey(threadsPerKey);
  }

  @Override
  protected void doInit(final Configuration configuration) throws Exception {
    super.doInit(configuration);
    if (cooperationBuilder != null) {
      indexCooperation = cooperationBuilder.build(getRepository().getName() + ":pypiIndex");
    }
  }

  public Content buildIndex(final String path, final Supplier<String> lazyMergeResult) throws IOException {
    if (isNull(indexCooperation)) {
      return buildMergedIndex(path, lazyMergeResult);
    }
    return indexCooperation.on(() -> buildMergedIndex(path, lazyMergeResult))
        .checkFunction(() -> Optional.ofNullable(getFromCache(path)).filter(content -> !isStale(content)))
        .cooperate(path);
  }

  private Content buildMergedIndex(final String path, final Supplier<String> lazyMergeResult) throws IOException {
    String html;
    try {
      html = lazyMergeResult.get();
    }
    catch (UncheckedIOException e) {
      throw e.getCause();
    }
    Content content = new Content(new StringPayload(html, TEXT_HTML));
    content.getAttributes().set(CONTENT_LAST_MODIFIED, DateTime.now());
    maintainCacheInfo(content.getAttributes());
    return saveToCache(path, content);
  }

  @Nullable
  public Content getFromCache(final String path) {
    return content().getAsset(path).map(FluentAsset::download).orElse(null);
  }

  public boolean isStale(final String path, @Nullable final Content content, final Map<Repository, Response> responses) {
    if (content == null || isStale(content)) {
      return true;
    }
    DateTime cacheModified = content.getAttributes().get(CONTENT_LAST_MODIFIED, DateTime.class);
    if (cacheModified == null) {
      return true;
    }
    for (Entry<Repository, Response> response : responses.entrySet()) {
      Response responseValue = response.getValue();
      if (responseValue.getStatus().getCode() == HttpStatus.OK && responseValue.getPayload() instanceof Content) {
        Content responseContent = (Content) responseValue.getPayload();
        DateTime memberLastModified = responseContent.getAttributes().get(CONTENT_LAST_MODIFIED, DateTime.class);
        if (memberLastModified == null || memberLastModified.isAfter(cacheModified)) {
          log.debug("Found stale content while fetching {} from repository {}", path, response.getKey().getName());
          return true;
        }
      }
    }
    return false;
  }

  private Content saveToCache(final String path, final Content content) {
    PypiContentFacet contentFacet = content();
    try (TempBlob tempBlob = contentFacet.blobs().ingest(content, HASH_ALGORITHMS)) {
      AssetKind kind = ContentPypiPathUtils.indexPath().equals(path) ? ROOT_INDEX : INDEX;
      return contentFacet.assets()
          .path(path)
          .kind(kind.name())
          .blob(tempBlob)
          .save()
          .markAsCached(content)
          .download();
    }
  }

  @Subscribe
  @AllowConcurrentEvents
  public void onAssetCreatedEvent(final AssetCreatedEvent event) {
    handleAssetEvent(event);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void onAssetUploadedEvent(final AssetUploadedEvent event) {
    handleAssetEvent(event);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void onAssetDeletedEvent(final AssetDeletedEvent event) {
    handleAssetEvent(event);
  }

  private void handleAssetEvent(final AssetEvent event) {
    if (!PyPiFormat.NAME.equals(event.getFormat())) {
      return;
    }
    Asset asset = event.getAsset();
    if (!INDEX.name().equals(asset.kind()) && !ROOT_INDEX.name().equals(asset.kind())) {
      return;
    }
    event.getRepository().ifPresent(repository -> {
      if (!repository.equals(getRepository()) && member(repository)) {
        invalidateCache(asset.path());
      }
    });
  }

  private void invalidateCache(final String path) {
    try {
      content().getAsset(path).ifPresent(cached -> {
        log.debug("Invalidating cached {} of {}", path, getRepository().getName());
        cached.markAsStale();
      });
    }
    catch (RuntimeException e) {
      log.debug("Unable to invalidate cached {} of {}", path, getRepository().getName(), e);
    }
  }

  private PypiContentFacet content() {
    return facet(PypiContentFacet.class);
  }
}
