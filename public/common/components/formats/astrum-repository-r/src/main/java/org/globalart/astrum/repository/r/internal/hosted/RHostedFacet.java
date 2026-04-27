
package org.globalart.astrum.repository.r.internal.hosted;

import java.util.Collection;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeSet;
import java.util.concurrent.locks.Lock;
import java.util.function.Consumer;

import javax.annotation.Nullable;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.repository.r.RContentFacet;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.r.internal.util.RPackagesBuilder;
import org.globalart.astrum.repository.r.internal.util.RPackagesUtils;
import org.globalart.astrum.repository.r.internal.util.RPathUtils;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.util.concurrent.Striped;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.repository.r.RContentFacet.ATTRIBUTES_KEY;
import static org.globalart.astrum.repository.r.internal.RAttributes.P_PACKAGE;
import static org.globalart.astrum.repository.r.internal.RAttributes.P_VERSION;
import static org.globalart.astrum.repository.r.internal.util.PackageValidator.validateArchiveFileName;
import static org.globalart.astrum.repository.r.internal.util.PackageValidator.validateArchiveUploadPath;
import static org.globalart.astrum.repository.r.internal.util.RDescriptionUtils.extractDescriptionFromArchive;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.directory;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.normalize;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.packagesPath;

@Facet.Exposed
@Component
@Qualifier(RFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RHostedFacet
    extends FacetSupport
{
  public static final String PACKAGE_COUNT = "packageCount";

  private static final String TEXT_PLAIN = "text/plain";

  private static final String APPLICATION_X_GZIP = "application/x-gzip";

  private final Striped<Lock> packagesLocks = Striped.lock(64);

  public Content upload(final String path, final Payload payload, final Consumer<String> authorizer) {
    String assetPath = normalize(path);
    validateArchiveUploadPath(assetPath);
    authorizer.accept(assetPath);
    RContentFacet content = content();
    try (TempBlob blob = content.ingest(payload)) {
      Map<String, String> description = extractDescriptionFromArchive(assetPath, blob::get);
      validateArchiveFileName(assetPath, description);
      Content stored = content.putPackage(assetPath, blob, description).download();
      rebuildPackages(directory(assetPath));
      return stored;
    }
  }

  public Optional<Content> getArchive(final String path) {
    return content().get(path);
  }

  public Optional<Content> getPackages(final String path) {
    Optional<Content> stored = content().get(path);
    if (stored.isPresent()) {
      return stored;
    }
    rebuildPackages(directory(path));
    return content().get(path);
  }

  public boolean delete(final String path) {
    Optional<FluentAsset> asset = content().getAsset(path);
    if (asset.isEmpty()) {
      return false;
    }
    facet(ContentMaintenanceFacet.class).deleteAsset(asset.get());
    return true;
  }

  public void rebuildPackages(final Collection<String> archivePaths) {
    for (String directory : new TreeSet<>(archivePaths.stream().map(RPathUtils::directory).toList())) {
      try {
        rebuildPackages(directory);
      }
      catch (RuntimeException e) {
        log.warn("Unable to rebuild PACKAGES in {} of repository {}", directory, getRepository().getName(), e);
      }
    }
  }

  @Nullable
  public Content rebuildPackages(final String directory) {
    Lock lock = packagesLocks.get(directory);
    lock.lock();
    try {
      RContentFacet content = content();
      List<FluentAsset> archives = content.archivesIn(directory);
      RPackagesBuilder builder = new RPackagesBuilder();
      for (FluentAsset archive : archives) {
        Map<String, ?> description = describe(archive);
        if (description != null) {
          String md5 = archive.blob().map(blob -> blob.checksums().get("md5")).orElse(null);
          builder.append(description, md5);
        }
      }
      if (builder.isEmpty()) {
        content.delete(packagesPath(directory, false));
        content.delete(packagesPath(directory, true));
        log.debug("Removed PACKAGES in {} of repository {}", directory, getRepository().getName());
        return null;
      }
      byte[] packages = builder.buildPackages().getBytes(UTF_8);
      Map<String, Object> attributes = new LinkedHashMap<>();
      attributes.put(PACKAGE_COUNT, builder.getPackageInformation().size());
      content.putMetadata(packagesPath(directory, true), RPackagesUtils.gzip(packages), APPLICATION_X_GZIP,
          attributes);
      Content plain = content.putMetadata(packagesPath(directory, false), packages, TEXT_PLAIN, attributes);
      log.debug("Rebuilt PACKAGES in {} of repository {} with {} packages", directory, getRepository().getName(),
          builder.getPackageInformation().size());
      return plain;
    }
    finally {
      lock.unlock();
    }
  }

  @Nullable
  private Map<String, ?> describe(final FluentAsset archive) {
    Map<String, Object> description = archive.attributes(ATTRIBUTES_KEY).backing();
    if (description.get(P_PACKAGE) != null && description.get(P_VERSION) != null) {
      return description;
    }
    try {
      Map<String, String> extracted = extractDescriptionFromArchive(archive.path(),
          () -> archive.download().openInputStream());
      archive.withAttribute(ATTRIBUTES_KEY, new LinkedHashMap<>(extracted));
      return extracted;
    }
    catch (RuntimeException e) {
      log.warn("Skipping unreadable R package {} of repository {}: {}", archive.path(), getRepository().getName(),
          e.getMessage());
      return null;
    }
  }

  private RContentFacet content() {
    return facet(RContentFacet.class);
  }
}
