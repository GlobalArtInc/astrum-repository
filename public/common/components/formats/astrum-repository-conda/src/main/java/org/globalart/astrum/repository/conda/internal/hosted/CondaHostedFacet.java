package org.globalart.astrum.repository.conda.internal.hosted;

import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.ConcurrentMap;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.Facet.Exposed;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.conda.AssetKind;
import org.globalart.astrum.repository.conda.CondaContentFacet;
import org.globalart.astrum.repository.conda.CondaFormat;
import org.globalart.astrum.repository.conda.internal.metadata.CondaPackageReader;
import org.globalart.astrum.repository.conda.internal.metadata.InvalidCondaPackageException;
import org.globalart.astrum.repository.conda.internal.metadata.Repodata;
import org.globalart.astrum.repository.conda.internal.metadata.Repodata.PackageRecord;
import org.globalart.astrum.repository.conda.util.CondaPathUtils;
import org.globalart.astrum.repository.conda.util.CondaPathUtils.PackagePath;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.hash.Hashing;
import org.joda.time.DateTime;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.conda.CondaContentFacet.INDEX_ATTRIBUTE;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.isValidSubdir;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.lastSegment;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.repodataPath;
import static org.globalart.astrum.repository.view.Content.CONTENT_ETAG;
import static org.globalart.astrum.repository.view.Content.CONTENT_LAST_MODIFIED;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

@Exposed
@Component
@Qualifier(CondaFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CondaHostedFacet
    extends FacetSupport
{
  private final ConcurrentMap<String, Object> repodataLocks = new ConcurrentHashMap<>();

  public Content upload(final String path, final Payload payload) throws IOException {
    PackagePath packagePath = CondaPathUtils.parsePackagePath(path)
        .orElseThrow(() -> new InvalidCondaPackageException("Invalid Conda package path " + path));
    try (TempBlob tempBlob = content().getTempBlob(payload)) {
      Map<String, Object> index = readIndex(tempBlob, packagePath.getAssetKind());
      validate(packagePath, index);
      return store(packagePath, tempBlob, index);
    }
  }

  public Map<String, Object> readIndex(final TempBlob tempBlob, final AssetKind assetKind) throws IOException {
    try (InputStream in = tempBlob.get()) {
      return CondaPackageReader.readIndex(in, assetKind);
    }
  }

  public PackagePath packagePath(final Map<String, Object> index, final AssetKind assetKind) throws IOException {
    String name = requiredField(index, "name");
    String version = requiredField(index, "version");
    String build = requiredField(index, "build");
    String subdir = requiredField(index, "subdir");
    if (!isValidSubdir(subdir)) {
      throw new InvalidCondaPackageException("Invalid subdir '" + subdir + "' in info/index.json");
    }
    return new PackagePath("/" + subdir, subdir, name, version, build, assetKind);
  }

  public Content store(final PackagePath packagePath, final TempBlob tempBlob, final Map<String, Object> index) {
    FluentAsset asset = content().putPackage(
        packagePath.getPath(),
        packagePath.getAssetKind(),
        packagePath.getSubdir(),
        packagePath.getName(),
        packagePath.getVersion(),
        tempBlob,
        index);
    rebuildRepodata(packagePath.getSubdirPath(), packagePath.getSubdir());
    return asset.download();
  }

  @Nullable
  public Content getPackage(final String path) {
    return content().getAsset(path).map(FluentAsset::download).orElse(null);
  }

  @Nullable
  public Content getRepodata(final String subdirPath, final String subdir) throws IOException {
    Optional<FluentAsset> asset = content().getAsset(repodataPath(subdirPath));
    if (asset.isPresent()) {
      return asset.get().download();
    }
    if (!isValidSubdir(subdir)) {
      return null;
    }
    byte[] empty = Repodata.build(subdir, List.of());
    Content content = new Content(new BytesPayload(empty, APPLICATION_JSON));
    content.getAttributes().set(CONTENT_LAST_MODIFIED, new DateTime(0L));
    content.getAttributes().set(CONTENT_ETAG, Hashing.sha1().hashBytes(empty).toString());
    return content;
  }

  public void rebuildRepodata(final String subdirPath, final String subdir) {
    Object lock = repodataLocks.computeIfAbsent(subdirPath, key -> new Object());
    synchronized (lock) {
      List<FluentAsset> packages = content().packagesIn(subdirPath);
      String repodataPath = repodataPath(subdirPath);
      if (packages.isEmpty()) {
        content().delete(repodataPath);
        log.debug("Removed {} of {}, no packages left", repodataPath, getRepository().getName());
        return;
      }
      List<PackageRecord> records = new ArrayList<>();
      for (FluentAsset asset : packages) {
        record(asset).ifPresent(records::add);
      }
      try (TempBlob tempBlob = content().getTempBlob(
          new BytesPayload(Repodata.build(subdir, records), APPLICATION_JSON))) {
        content().putMetadata(repodataPath, AssetKind.ARCH_REPODATA_JSON, tempBlob);
      }
      catch (IOException e) {
        throw new IllegalStateException("Unable to build " + repodataPath + " of " + getRepository().getName(), e);
      }
      log.debug("Rebuilt {} of {} with {} packages", repodataPath, getRepository().getName(), records.size());
    }
  }

  private Optional<PackageRecord> record(final FluentAsset asset) {
    Optional<AssetBlob> blob = asset.blob();
    if (blob.isEmpty()) {
      return Optional.empty();
    }
    NestedAttributesMap attributes = asset.attributes(CondaFormat.NAME);
    Object index = attributes.get(INDEX_ATTRIBUTE);
    if (!(index instanceof Map)) {
      log.warn("Conda package {} of {} has no index metadata, skipping it", asset.path(), getRepository().getName());
      return Optional.empty();
    }
    Map<String, Object> fields = new LinkedHashMap<>();
    ((Map<?, ?>) index).forEach((key, value) -> fields.put(String.valueOf(key), value));
    Map<String, String> checksums = blob.get().checksums();
    fields.put("md5", checksums.get(HashAlgorithm.MD5.name()));
    fields.put("sha256", checksums.get(HashAlgorithm.SHA256.name()));
    fields.put("size", blob.get().blobSize());
    return Optional.of(new PackageRecord(
        lastSegment(asset.path()), AssetKind.valueOf(asset.kind()), fields));
  }

  private static void validate(final PackagePath packagePath, final Map<String, Object> index) throws IOException {
    String name = requiredField(index, "name");
    String version = requiredField(index, "version");
    String build = requiredField(index, "build");
    if (!isValidSubdir(packagePath.getSubdir())) {
      throw new InvalidCondaPackageException("Invalid subdir '" + packagePath.getSubdir() + "' in upload path");
    }
    Object subdir = index.get("subdir");
    if (subdir != null && !packagePath.getSubdir().equals(subdir)) {
      throw new InvalidCondaPackageException(
          "Package subdir '" + subdir + "' does not match upload path subdir '" + packagePath.getSubdir() + "'");
    }
    if (!packagePath.getName().equals(name) || !packagePath.getVersion().equals(version) ||
        !packagePath.getBuild().equals(build)) {
      throw new InvalidCondaPackageException("File name " + packagePath.getFileName() +
          " does not match package " + name + "-" + version + "-" + build);
    }
  }

  private static String requiredField(final Map<String, Object> index, final String field) throws IOException {
    Object value = index.get(field);
    if (value == null || String.valueOf(value).isBlank()) {
      throw new InvalidCondaPackageException("info/index.json is missing '" + field + "'");
    }
    return String.valueOf(value);
  }

  private CondaContentFacet content() {
    return facet(CondaContentFacet.class);
  }
}
