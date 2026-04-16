package org.globalart.astrum.repository.conda.internal.group;

import java.io.IOException;
import java.io.InputStream;
import java.time.Duration;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.cooperation2.Cooperation2;
import org.globalart.astrum.common.cooperation2.Cooperation2Factory;
import org.globalart.astrum.common.event.EventAware;
import org.globalart.astrum.repository.Facet.Exposed;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.Type;
import org.globalart.astrum.repository.cache.RepositoryCacheInvalidationService;
import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.CondaContentFacet;
import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.conda.internal.metadata.Repodata;
import org.globalart.astrum.repository.config.Configuration;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.event.asset.AssetCreatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetDeletedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetEvent;
import org.globalart.astrum.repository.content.event.asset.AssetUpdatedEvent;
import org.globalart.astrum.repository.content.event.asset.AssetUploadedEvent;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.group.GroupFacetImpl;
import org.globalart.astrum.repository.manager.RepositoryManager;
import org.globalart.astrum.repository.types.GroupType;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
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

import static org.globalart.astrum.repository.conda.util.CondaPathUtils.REPODATA_JSON;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.REPODATA_JSON_ZST;
import static org.globalart.astrum.repository.view.Content.CONTENT_LAST_MODIFIED;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

@Exposed
@Component
@Qualifier(CondaFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CondaGroupFacet
    extends GroupFacetImpl
    implements EventAware.Asynchronous
{
  @Nullable
  private Cooperation2Factory.Builder cooperationBuilder;

  @Nullable
  private Cooperation2 repodataCooperation;

  @Autowired
  public CondaGroupFacet(
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
      @Value("${astrum.conda.repodata.cooperation.enabled:true}") final boolean cooperationEnabled,
      @Value("${astrum.conda.repodata.cooperation.majorTimeout:0s}") final Duration majorTimeout,
      @Value("${astrum.conda.repodata.cooperation.minorTimeout:120s}") final Duration minorTimeout,
      @Value("${astrum.conda.repodata.cooperation.threadsPerKey:100}") final int threadsPerKey)
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
      repodataCooperation = cooperationBuilder.build(getRepository().getName() + ":condaRepodata");
    }
  }

  public Content buildRepodata(final String path, final String subdir, final List<Repodata.Source> sources)
      throws IOException
  {
    if (repodataCooperation == null) {
      return mergeRepodata(path, subdir, sources);
    }
    return repodataCooperation.on(() -> mergeRepodata(path, subdir, sources))
        .checkFunction(() -> Optional.ofNullable(getFromCache(path)).filter(content -> !isStale(content)))
        .cooperate(path);
  }

  private Content mergeRepodata(final String path, final String subdir, final List<Repodata.Source> sources)
      throws IOException
  {
    long start = System.currentTimeMillis();
    CondaContentFacet content = content();
    try (InputStream merged = Repodata.merge(subdir, sources);
         TempBlob tempBlob = content.getTempBlob(merged, APPLICATION_JSON)) {
      Content marker = new Content(new BytesPayload(new byte[0], APPLICATION_JSON));
      marker.getAttributes().set(CONTENT_LAST_MODIFIED, DateTime.now());
      maintainCacheInfo(marker.getAttributes());
      Content result = content.putMetadata(path, AssetKind.ARCH_REPODATA_JSON, tempBlob)
          .markAsCached(marker)
          .download();
      log.debug("Merged {} of {} from {} members in {} ms", path, getRepository().getName(), sources.size(),
          System.currentTimeMillis() - start);
      return result;
    }
  }

  @Nullable
  public Content getFromCache(final String path) {
    return content().getAsset(path).map(FluentAsset::download).orElse(null);
  }

  public boolean isStale(@Nullable final Content cached, final Map<Repository, Response> responses) {
    if (cached == null || isStale(cached)) {
      return true;
    }
    DateTime cacheModified = cached.getAttributes().get(CONTENT_LAST_MODIFIED, DateTime.class);
    if (cacheModified == null) {
      return true;
    }
    for (Map.Entry<Repository, Response> entry : responses.entrySet()) {
      if (entry.getValue().getPayload() instanceof Content) {
        Content memberContent = (Content) entry.getValue().getPayload();
        DateTime memberModified = memberContent.getAttributes().get(CONTENT_LAST_MODIFIED, DateTime.class);
        if (memberModified == null || memberModified.isAfter(cacheModified)) {
          log.debug("Repodata of member {} is newer than the merged repodata of {}", entry.getKey().getName(),
              getRepository().getName());
          return true;
        }
      }
    }
    return false;
  }

  @Subscribe
  @AllowConcurrentEvents
  public void onAssetCreated(final AssetCreatedEvent event) {
    handleAssetEvent(event);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void onAssetUploaded(final AssetUploadedEvent event) {
    handleAssetEvent(event);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void onAssetUpdated(final AssetUpdatedEvent event) {
    handleAssetEvent(event);
  }

  @Subscribe
  @AllowConcurrentEvents
  public void onAssetDeleted(final AssetDeletedEvent event) {
    handleAssetEvent(event);
  }

  private void handleAssetEvent(final AssetEvent event) {
    if (!CondaFormat.NAME.equals(event.getFormat())) {
      return;
    }
    Asset asset = event.getAsset();
    if (!AssetKind.isRepodata(asset.kind())) {
      return;
    }
    event.getRepository().ifPresent(repository -> {
      if (!repository.equals(getRepository()) && member(repository)) {
        invalidate(toRepodataPath(asset.path()));
      }
    });
  }

  private static String toRepodataPath(final String path) {
    if (path.endsWith(REPODATA_JSON_ZST)) {
      return path.substring(0, path.length() - REPODATA_JSON_ZST.length()) + REPODATA_JSON;
    }
    return path;
  }

  private void invalidate(final String path) {
    try {
      content().getAsset(path).ifPresent(cached -> {
        log.debug("Invalidating merged {} of {}", path, getRepository().getName());
        cached.markAsStale();
      });
    }
    catch (RuntimeException e) {
      log.debug("Unable to invalidate merged {} of {}", path, getRepository().getName(), e);
    }
  }

  private CondaContentFacet content() {
    return facet(CondaContentFacet.class);
  }
}
