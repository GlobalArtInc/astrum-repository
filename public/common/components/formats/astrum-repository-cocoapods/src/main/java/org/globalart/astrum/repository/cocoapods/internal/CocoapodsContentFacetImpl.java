package org.globalart.astrum.repository.cocoapods.internal;

import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import javax.annotation.Nullable;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.cocoapods.CocoapodsContentFacet;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.google.common.base.Strings;
import com.google.common.collect.ImmutableList;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;

@Component
@Qualifier(CocoapodsFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class CocoapodsContentFacetImpl
    extends ContentFacetSupport
    implements CocoapodsContentFacet
{
  public static final List<HashAlgorithm> HASH_ALGORITHMS =
      ImmutableList.of(HashAlgorithm.SHA1, HashAlgorithm.SHA256, HashAlgorithm.MD5);

  @Autowired
  public CocoapodsContentFacetImpl(@Qualifier(CocoapodsFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    return assets().path(normalize(path)).find();
  }

  @Override
  public FluentAsset putMetadata(final String path, final TempBlob tempBlob) {
    return assets()
        .path(normalize(path))
        .kind(AssetKind.CDN_METADATA.name())
        .blob(checkNotNull(tempBlob))
        .save();
  }

  @Override
  public FluentAsset putSpec(
      final String path,
      final TempBlob tempBlob,
      @Nullable final String name,
      @Nullable final String version,
      @Nullable final String remoteUrl)
  {
    Map<String, Object> attributes = new HashMap<>();
    if (name != null) {
      attributes.put(POD_NAME_ATTRIBUTE, name);
    }
    if (version != null) {
      attributes.put(POD_VERSION_ATTRIBUTE, version);
    }
    attributes.put(POD_REMOTE_ATTRIBUTE, Strings.nullToEmpty(remoteUrl));
    return assets()
        .path(normalize(path))
        .kind(AssetKind.SPEC.name())
        .attributes(CocoapodsFormat.NAME, attributes)
        .blob(checkNotNull(tempBlob))
        .save();
  }

  @Override
  public FluentAsset putPod(
      final String path,
      final String name,
      final String version,
      final String remoteUrl,
      final TempBlob tempBlob)
  {
    FluentComponent component = components()
        .name(checkNotNull(name))
        .version(checkNotNull(version))
        .getOrCreate();
    return component.asset(normalize(path))
        .kind(AssetKind.POD.name())
        .attributes(CocoapodsFormat.NAME, Map.of(POD_REMOTE_ATTRIBUTE, checkNotNull(remoteUrl)))
        .blob(checkNotNull(tempBlob))
        .save();
  }

  @Nullable
  @Override
  public String getPodRemoteUrl(final String specPath) {
    return getAsset(specPath)
        .map(asset -> asset.attributes(CocoapodsFormat.NAME).get(POD_REMOTE_ATTRIBUTE, String.class))
        .map(Strings::emptyToNull)
        .orElse(null);
  }

  @Override
  public TempBlob getTempBlob(final Payload payload) {
    return blobs().ingest(checkNotNull(payload), HASH_ALGORITHMS);
  }

  private static String normalize(final String path) {
    checkNotNull(path);
    return path.startsWith("/") ? path : "/" + path;
  }
}
