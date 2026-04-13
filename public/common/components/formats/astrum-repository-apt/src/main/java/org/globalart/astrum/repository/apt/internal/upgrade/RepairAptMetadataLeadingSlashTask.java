
package org.globalart.astrum.repository.apt.internal.upgrade;

import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.stream.Stream;

import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.logging.task.ProgressLogIntervalHelper;
import org.globalart.astrum.logging.task.TaskLogging;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.RepositoryParallelTaskSupport;
import org.globalart.astrum.repository.apt.AptFormat;
import org.globalart.astrum.repository.apt.datastore.AptContentFacet;
import org.globalart.astrum.repository.apt.datastore.internal.hosted.AptHostedFacet;
import org.globalart.astrum.repository.apt.internal.AptProperties;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AttributeOperation;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.types.HostedType;
import org.globalart.astrum.scheduling.Cancelable;
import org.globalart.astrum.scheduling.CancelableHelper;

import com.google.common.annotations.VisibleForTesting;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.logging.task.TaskLogType.ASTRUM_LOG_ONLY;

/**
 * Updates apt DEB asset attributes to remove leading slash from apt metadata and triggers rebuilding metadata
 */
@Component
@TaskLogging(ASTRUM_LOG_ONLY)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RepairAptMetadataLeadingSlashTask
    extends RepositoryParallelTaskSupport
    implements Cancelable
{
  @VisibleForTesting
  static final String INDEX_SECTION = "index_section";

  private static final String BAD_PATTERN = "Filename: /";

  private final Map<String, Repository> repositoryRequiringRebuild = new ConcurrentHashMap<>();

  private final AtomicInteger count = new AtomicInteger();

  @Autowired
  public RepairAptMetadataLeadingSlashTask(
      @Value("${external.metadata.repository.concurrencyLimit:5}") final int concurrencyLimit)
  {
    super(false, concurrencyLimit);
  }

  @Override
  protected Stream<Runnable> jobStream(final ProgressLogIntervalHelper progress, final Repository repository) {
    AptContentFacet content = repository.facet(AptContentFacet.class);
    return Continuations
        .streamOf(
            content.assets().byFilter("kind = #{filterParams.kind}", Map.of("kind", AptProperties.DEB))::browse)
        .filter(this::hasLeadingSlash)
        .map(asset -> createJob(progress, asset));
  }

  private Runnable createJob(final ProgressLogIntervalHelper progress, final FluentAsset asset) {
    return () -> {
      CancelableHelper.checkCancellation();
      Repository repository = asset.repository();
      log.debug("Processing {} in {}", asset, repository);
      try {
        String indexSection = asset.attributes(AptFormat.NAME).get(INDEX_SECTION, String.class);
        asset.attributes(AttributeOperation.OVERLAY, AptFormat.NAME,
            Map.of(INDEX_SECTION, indexSection.replace(BAD_PATTERN, "Filename: ")));

        repositoryRequiringRebuild.putIfAbsent(repository.getName(), repository);

        progress.info("Updated {} assets with leading slash in metadata", count.incrementAndGet());
      }
      catch (Exception e) {
        log.error("Failed to update Filename path for {} in {}", asset.path(), asset.repository().getName(), e);
      }
    };
  }

  @Override
  protected Object result() {
    for (Repository repository : repositoryRequiringRebuild.values()) {
      CancelableHelper.checkCancellation();
      try {
        log.info("Rebuilding repository metadata for {}", repository.getName());
        repository.facet(AptHostedFacet.class).rebuildMetadata();
      }
      catch (Exception e) {
        log.error("Failed to rebuild metadata for repository {}", repository.getName(), e);
      }
    }
    return null;
  }

  private boolean hasLeadingSlash(final Asset asset) {
    return Optional.ofNullable(asset.attributes(AptFormat.NAME).get(INDEX_SECTION, String.class))
        .map(indexSection -> indexSection.contains(BAD_PATTERN))
        .orElse(false);
  }

  @Override
  protected boolean appliesTo(final Repository repository) {
    return repository.getFormat() instanceof AptFormat && repository.getType() instanceof HostedType;
  }

  @Override
  public String getMessage() {
    return RepairAptMetadataLeadingSlashTaskDescriptor.TASK_DESCRIPTION;
  }
}
