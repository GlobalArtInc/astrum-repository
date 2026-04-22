package org.globalart.astrum.repository.alpine.internal.hosted;

import java.io.IOException;
import java.io.InputStream;
import java.time.Instant;
import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.Collection;
import java.util.Comparator;
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
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.alpine.AlpineContentFacet;
import org.globalart.astrum.repository.alpine.AlpineFormat;
import org.globalart.astrum.repository.alpine.internal.AlpinePathUtils;
import org.globalart.astrum.repository.alpine.internal.AlpineSigningFacet;
import org.globalart.astrum.repository.alpine.internal.apk.ApkIndexArchive;
import org.globalart.astrum.repository.alpine.internal.apk.ApkIndexEntry;
import org.globalart.astrum.repository.alpine.internal.apk.ApkPackage;
import org.globalart.astrum.repository.alpine.internal.apk.ApkPackageParser;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.maintenance.ContentMaintenanceFacet;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.util.concurrent.Striped;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.alpine.AlpineContentFacet.ATTRIBUTES_KEY;
import static org.globalart.astrum.repository.alpine.internal.AlpinePathUtils.directory;
import static org.globalart.astrum.repository.alpine.internal.AlpinePathUtils.indexPath;

@Facet.Exposed
@Component
@Qualifier(AlpineFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class AlpineHostedFacet
    extends FacetSupport
{
  public static final String SIGNER = "signer";

  public static final String PACKAGE_COUNT = "packageCount";

  private final Striped<Lock> indexLocks = Striped.lock(64);

  public Content put(
      final String directory,
      @Nullable final String fileName,
      final Payload payload,
      final Consumer<String> authorizer) throws IOException
  {
    if (directory.isEmpty() || "/".equals(directory)) {
      throw new InvalidContentException(
          "Alpine packages must be stored in an architecture directory such as /v3.20/main/x86_64");
    }
    AlpineContentFacet content = content();
    try (TempBlob blob = content.ingest(payload)) {
      ApkPackage apkPackage = parse(blob);
      if (fileName != null && !apkPackage.fileName().equals(fileName)) {
        throw new InvalidContentException(
            "Package file name " + fileName + " does not match its metadata, expected " + apkPackage.fileName());
      }
      String architecture = AlpinePathUtils.lastSegment(directory);
      if (apkPackage.arch() != null && !apkPackage.isNoarch() && !apkPackage.arch().equals(architecture)) {
        throw new InvalidContentException("Package " + apkPackage.fileName() + " is built for " + apkPackage.arch()
            + " and cannot be stored in the " + architecture + " directory");
      }
      String path = directory + "/" + apkPackage.fileName();
      authorizer.accept(path);
      Content stored = content.putPackage(path, blob, apkPackage);
      rebuildIndex(directory);
      return stored;
    }
  }

  public Optional<Content> getPackage(final String path) {
    return content().get(path);
  }

  public Optional<Content> getIndex(final String directory) throws IOException {
    Optional<FluentAsset> index = content().getAsset(indexPath(directory));
    if (index.isPresent() && signing().signerId().equals(signer(index.get()))) {
      return Optional.of(index.get().download());
    }
    return Optional.ofNullable(rebuildIndex(directory));
  }

  public boolean delete(final String path) {
    Optional<FluentAsset> asset = content().getAsset(path);
    if (asset.isEmpty()) {
      return false;
    }
    facet(ContentMaintenanceFacet.class).deleteAsset(asset.get());
    return true;
  }

  public void rebuildIndexes(final Collection<String> packagePaths) {
    for (String directory : new TreeSet<>(packagePaths.stream().map(AlpinePathUtils::directory).toList())) {
      try {
        rebuildIndex(directory);
      }
      catch (IOException | RuntimeException e) {
        log.warn("Unable to rebuild {} of repository {}", indexPath(directory), getRepository().getName(), e);
      }
    }
  }

  @Nullable
  public Content rebuildIndex(final String directory) throws IOException {
    Lock lock = indexLocks.get(directory);
    lock.lock();
    try {
      AlpineContentFacet content = content();
      List<FluentAsset> packages = content.packagesIn(directory);
      if (packages.isEmpty()) {
        content.delete(indexPath(directory));
        return null;
      }
      List<ApkIndexEntry> entries = new ArrayList<>();
      Instant modified = Instant.EPOCH;
      for (FluentAsset asset : packages) {
        ApkPackage apkPackage = packageOf(asset);
        if (apkPackage != null) {
          entries.add(apkPackage.indexEntry());
        }
        OffsetDateTime updated = asset.lastUpdated();
        if (updated != null && updated.toInstant().isAfter(modified)) {
          modified = updated.toInstant();
        }
      }
      entries.sort(Comparator.comparing(ApkIndexEntry::name).thenComparing(ApkIndexEntry::version));
      AlpineSigningFacet signing = signing();
      byte[] index = ApkIndexArchive.build(getRepository().getName(), entries, signing.signingKey(), modified);
      Map<String, Object> attributes = new LinkedHashMap<>();
      attributes.put(SIGNER, signing.signerId());
      attributes.put(PACKAGE_COUNT, entries.size());
      log.debug("Rebuilt {} of repository {} with {} packages", indexPath(directory), getRepository().getName(),
          entries.size());
      return content.putIndex(indexPath(directory), index, attributes);
    }
    finally {
      lock.unlock();
    }
  }

  @Nullable
  private ApkPackage packageOf(final FluentAsset asset) {
    long size = asset.blob().map(blob -> blob.blobSize()).orElse(0L);
    Map<String, Object> attributes = asset.attributes(ATTRIBUTES_KEY).backing();
    if (attributes.get(ApkPackage.CHECKSUM) != null && attributes.get(ApkPackage.NAME) != null) {
      return ApkPackage.fromAttributes(attributes, size);
    }
    try (InputStream in = asset.download().openInputStream()) {
      ApkPackage apkPackage = ApkPackageParser.parse(in, size);
      asset.withAttribute(ATTRIBUTES_KEY, apkPackage.toAttributes());
      return apkPackage;
    }
    catch (IOException | IllegalArgumentException e) {
      log.warn("Skipping unreadable package {} of repository {}: {}", asset.path(), getRepository().getName(),
          e.getMessage());
      return null;
    }
  }

  private ApkPackage parse(final TempBlob blob) {
    try (InputStream in = blob.get()) {
      return ApkPackageParser.parse(in, blob.getBlob().getMetrics().getContentSize());
    }
    catch (IOException e) {
      throw new InvalidContentException("Not a valid Alpine package: " + e.getMessage());
    }
    catch (IllegalArgumentException e) {
      throw new InvalidContentException(e.getMessage());
    }
  }

  @Nullable
  private static String signer(final FluentAsset asset) {
    return asset.attributes(ATTRIBUTES_KEY).get(SIGNER, String.class);
  }

  private AlpineContentFacet content() {
    return facet(AlpineContentFacet.class);
  }

  private AlpineSigningFacet signing() {
    return facet(AlpineSigningFacet.class);
  }
}
