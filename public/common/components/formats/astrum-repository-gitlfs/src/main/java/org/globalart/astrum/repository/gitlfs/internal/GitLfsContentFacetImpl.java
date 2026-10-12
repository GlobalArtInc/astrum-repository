package org.globalart.astrum.repository.gitlfs.internal;

import java.util.List;
import java.util.Optional;
import java.util.OptionalLong;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.gitlfs.GitLfsFormat;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(GitLfsFormat.NAME)
public class GitLfsContentFacetImpl
    extends ContentFacetSupport
    implements GitLfsContentFacet
{
  private static final List<HashAlgorithm> HASH_ALGORITHMS = List.of(SHA1, SHA256);

  @Autowired
  public GitLfsContentFacetImpl(@Qualifier(GitLfsFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  public Optional<FluentAsset> getObject(final String oid) {
    if (!GitLfsObjects.isValidOid(oid)) {
      return Optional.empty();
    }
    return assets().path(GitLfsObjects.assetPath(oid)).find();
  }

  @Override
  public OptionalLong size(final String oid) {
    return getObject(oid)
        .flatMap(FluentAsset::blob)
        .map(AssetBlob::blobSize)
        .map(OptionalLong::of)
        .orElseGet(OptionalLong::empty);
  }

  @Override
  public FluentAsset put(final String oid, final Payload payload) {
    checkNotNull(payload);
    try (TempBlob blob = ingest(payload)) {
      String actual = sha256(blob);
      if (!oid.equals(actual)) {
        throw GitLfsException.unprocessable("Object content hashes to " + actual + ", not to " + oid);
      }
      return put(blob);
    }
  }

  @Override
  public TempBlob ingest(final Payload payload) {
    return blobs().ingest(payload, HASH_ALGORITHMS);
  }

  @Override
  public boolean isReadOnly() {
    return getConfiguredWritePolicy() == WritePolicy.DENY;
  }

  @Override
  public FluentAsset put(final TempBlob blob) {
    String oid = sha256(blob);
    Optional<FluentAsset> existing = getObject(oid);
    if (existing.isPresent()) {
      return existing.get();
    }
    return assets()
        .path(GitLfsObjects.assetPath(oid))
        .kind(OBJECT_KIND)
        .component(components().name(oid).getOrCreate())
        .blob(blob)
        .save();
  }

  private static String sha256(final TempBlob blob) {
    return blob.getHashes().get(SHA256).toString();
  }
}
