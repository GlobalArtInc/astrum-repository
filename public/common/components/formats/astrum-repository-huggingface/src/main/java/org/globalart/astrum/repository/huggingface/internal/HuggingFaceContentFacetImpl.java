package org.globalart.astrum.repository.huggingface.internal;

import java.util.ArrayList;
import java.util.Collection;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.TreeSet;

import org.globalart.astrum.blobstore.api.Blob;
import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.huggingface.HuggingFaceFormat;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.hash.HashCode;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.repository.content.AttributeOperation.SET;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.ATTRIBUTES;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.COMMIT;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.ETAG;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.FILE;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.LFS;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.REPO_ID;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.REPO_TYPE;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.REVISIONS;
import static org.globalart.astrum.repository.huggingface.internal.HuggingFaceAttributes.SIZE;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(HuggingFaceFormat.NAME)
public class HuggingFaceContentFacetImpl
    extends ContentFacetSupport
    implements HuggingFaceContentFacet
{
  private static final List<HashAlgorithm> HASH_ALGORITHMS = List.of(SHA1, SHA256);

  private static final String PATH_LIKE_FILTER = "path LIKE #{filterParams.pattern}";

  private static final int REUSE_CANDIDATES = 20;

  @Autowired
  public HuggingFaceContentFacetImpl(@Qualifier(HuggingFaceFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    return assets().path(checkNotNull(path)).find();
  }

  @Override
  public FluentAsset saveRevision(final String path, final byte[] json) {
    try (TempBlob blob = blobs().ingest(new BytesPayload(json, APPLICATION_JSON), HASH_ALGORITHMS)) {
      return assets()
          .path(path)
          .kind(AssetKind.REVISION.name())
          .blob(blob)
          .save();
    }
  }

  @Override
  public FluentAsset saveFile(final HuggingFaceFile file, final Payload content) {
    try (TempBlob blob = blobs().ingest(content, HASH_ALGORITHMS)) {
      verify(file, blob);
      return fileBuilder(file, blob.getBlob().getMetrics().getContentSize()).blob(blob).save();
    }
  }

  @Override
  public Optional<FluentAsset> reuseFile(final HuggingFaceFile file) {
    if (file.etag() == null) {
      return Optional.empty();
    }
    String prefix = "/" + file.type().plural() + "/" + file.repoId() + "/";
    String suffix = "/" + file.path();
    Map<String, Object> params = Map.of("pattern", escapeLike(prefix) + "%" + escapeLike(suffix));
    Continuation<FluentAsset> candidates = assets().byFilter(PATH_LIKE_FILTER, params).browse(REUSE_CANDIDATES, null);
    for (FluentAsset candidate : candidates) {
      String path = candidate.path();
      String commit = path.length() > prefix.length() + suffix.length()
          ? path.substring(prefix.length(), path.length() - suffix.length()) : "";
      if (!path.startsWith(prefix) || !path.endsWith(suffix) || !HuggingFacePaths.isCommit(commit)
          || commit.equals(file.commit()) || !file.etag().equals(candidate.attributes(ATTRIBUTES).get(ETAG))) {
        continue;
      }
      Optional<AssetBlob> assetBlob = candidate.blob();
      Optional<Blob> blob = assetBlob.flatMap(found -> blobs().blob(found.blobRef()));
      if (blob.isEmpty() || (file.size() >= 0 && file.size() != assetBlob.get().blobSize())) {
        continue;
      }
      return Optional.of(fileBuilder(file, assetBlob.get().blobSize())
          .blob(blob.get(), checksums(assetBlob.get()))
          .save());
    }
    return Optional.empty();
  }

  private FluentAssetBuilder fileBuilder(final HuggingFaceFile file, final long size) {
    FluentComponent component = components()
        .name(file.repoId())
        .namespace(file.type().plural())
        .kind(file.type().kind())
        .version(file.commit())
        .getOrCreate();
    Map<String, Object> componentAttributes = new LinkedHashMap<>();
    componentAttributes.put(REPO_ID, file.repoId());
    componentAttributes.put(REPO_TYPE, file.type().kind());
    componentAttributes.put(COMMIT, file.commit());
    componentAttributes.put(REVISIONS, mergeRevisions(component, file));
    component.attributes(SET, ATTRIBUTES, componentAttributes);

    Map<String, Object> assetAttributes = new LinkedHashMap<>(componentAttributes);
    assetAttributes.put(FILE, file.path());
    assetAttributes.put(LFS, file.lfsSha256() != null);
    if (file.etag() != null) {
      assetAttributes.put(ETAG, file.etag());
    }
    assetAttributes.put(SIZE, size);
    return assets()
        .path(file.assetPath())
        .kind(AssetKind.FILE.name())
        .component(component)
        .attributes(ATTRIBUTES, assetAttributes);
  }

  private static Map<HashAlgorithm, HashCode> checksums(final AssetBlob assetBlob) {
    Map<HashAlgorithm, HashCode> checksums = new HashMap<>();
    assetBlob.checksums().forEach((name, value) -> HashAlgorithm.getHashAlgorithm(name)
        .ifPresent(algorithm -> checksums.put(algorithm, HashCode.fromString(value))));
    return checksums;
  }

  private static String escapeLike(final String value) {
    return value.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_");
  }

  private static void verify(final HuggingFaceFile file, final TempBlob blob) {
    if (file.lfsSha256() != null) {
      String actual = String.valueOf(blob.getHashes().get(SHA256));
      if (!file.lfsSha256().equals(actual)) {
        throw new InvalidContentException(String.format("%s of %s at %s has SHA-256 %s, the Hub announced %s",
            file.path(), file.repoId(), file.commit(), actual, file.lfsSha256()));
      }
    }
    long size = blob.getBlob().getMetrics().getContentSize();
    if (file.size() >= 0 && file.size() != size) {
      throw new InvalidContentException(String.format("%s of %s at %s has %d bytes, the Hub announced %d",
          file.path(), file.repoId(), file.commit(), size, file.size()));
    }
  }

  private static List<String> mergeRevisions(final FluentComponent component, final HuggingFaceFile file) {
    TreeSet<String> revisions = new TreeSet<>(file.revisions());
    Object attributes = component.attributes().get(ATTRIBUTES);
    if (attributes instanceof Map<?, ?> map && map.get(REVISIONS) instanceof Collection<?> known) {
      known.forEach(revision -> revisions.add(String.valueOf(revision)));
    }
    return new ArrayList<>(revisions);
  }
}
