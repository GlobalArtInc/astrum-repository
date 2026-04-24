
package org.globalart.astrum.repository.r.internal;

import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;

import org.globalart.astrum.repository.InvalidContentException;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentAssetBuilder;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.r.AssetKind;
import org.globalart.astrum.repository.r.RContentFacet;
import org.globalart.astrum.repository.r.RFormat;
import org.globalart.astrum.repository.r.internal.util.RDescriptionUtils;
import org.globalart.astrum.repository.r.internal.util.RPathUtils;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.entity.Continuations.iterableOf;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW;
import static org.globalart.astrum.repository.r.AssetKind.ARCHIVE;
import static org.globalart.astrum.repository.r.internal.RAttributes.P_PACKAGE;
import static org.globalart.astrum.repository.r.internal.RAttributes.P_VERSION;
import static org.globalart.astrum.repository.r.internal.util.RMetadataUtils.HASH_ALGORITHMS;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.fileName;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.namespace;
import static org.globalart.astrum.repository.r.internal.util.RPathUtils.normalize;

@Component
@Qualifier(RFormat.NAME)
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class RContentFacetImpl
    extends ContentFacetSupport
    implements RContentFacet
{
  private static final String ARCHIVES_FILTER = "kind = #{filterParams.kind} AND path LIKE #{filterParams.prefix}";

  @Autowired
  public RContentFacetImpl(@Qualifier(RFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    if (!ARCHIVE.name().equals(asset.kind())) {
      return ALLOW;
    }
    return super.writePolicy(asset);
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    return assets().path(normalize(checkNotNull(path))).find();
  }

  @Override
  public Optional<Content> get(final String path) {
    return getAsset(path).map(FluentAsset::download);
  }

  @Override
  public TempBlob ingest(final Payload payload) {
    return blobs().ingest(checkNotNull(payload), HASH_ALGORITHMS);
  }

  @Override
  public FluentAsset putPackage(final String path, final TempBlob blob, final Map<String, String> description) {
    checkNotNull(path);
    checkNotNull(blob);
    FluentComponent component = component(path, description.get(P_PACKAGE), description.get(P_VERSION));
    FluentAsset asset = assets()
        .path(normalize(path))
        .kind(ARCHIVE.name())
        .component(component)
        .blob(blob)
        .save();
    return asset.withAttribute(ATTRIBUTES_KEY, new LinkedHashMap<>(description));
  }

  @Override
  public Content putProxied(final String path, final AssetKind assetKind, final Content content) {
    try (TempBlob blob = ingest(content)) {
      FluentAssetBuilder builder = assets().path(normalize(path)).kind(assetKind.name()).blob(blob);
      Map<String, String> description = null;
      if (assetKind == ARCHIVE) {
        description = describe(path, blob);
        String name = description != null ? description.get(P_PACKAGE) : null;
        String version = description != null ? description.get(P_VERSION) : null;
        if (name == null || version == null) {
          Optional<String[]> coordinates = RPathUtils.nameAndVersion(fileName(path));
          name = coordinates.map(value -> value[0]).orElse(null);
          version = coordinates.map(value -> value[1]).orElse(null);
        }
        if (name != null && version != null) {
          builder.component(component(path, name, version));
        }
      }
      FluentAsset asset = builder.save();
      if (description != null) {
        asset = asset.withAttribute(ATTRIBUTES_KEY, new LinkedHashMap<>(description));
      }
      return asset.markAsCached(content).download();
    }
  }

  @Override
  public Content putMetadata(
      final String path,
      final byte[] bytes,
      final String contentType,
      final Map<String, Object> attributes)
  {
    try (TempBlob blob = blobs().ingest(new BytesPayload(bytes, contentType), HASH_ALGORITHMS)) {
      FluentAsset asset = assets()
          .path(normalize(path))
          .kind(AssetKind.PACKAGES.name())
          .blob(blob)
          .save();
      if (!attributes.isEmpty()) {
        asset = asset.withAttribute(ATTRIBUTES_KEY, attributes);
      }
      return asset.download();
    }
  }

  @Override
  public boolean delete(final String path) {
    return getAsset(path).map(FluentAsset::delete).orElse(false);
  }

  @Override
  public List<FluentAsset> archivesIn(final String directory) {
    String prefix = normalize(directory) + "/";
    FluentQuery<FluentAsset> query = assets().byFilter(ARCHIVES_FILTER,
        Map.of("kind", ARCHIVE.name(), "prefix", prefix + "%"));
    List<FluentAsset> archives = new ArrayList<>();
    for (FluentAsset asset : iterableOf(query::browse)) {
      String path = asset.path();
      if (path.startsWith(prefix) && path.indexOf('/', prefix.length()) < 0) {
        archives.add(asset);
      }
    }
    return archives;
  }

  private FluentComponent component(final String path, final String name, final String version) {
    return components()
        .name(checkNotNull(name))
        .namespace(namespace(path))
        .version(checkNotNull(version))
        .getOrCreate();
  }

  private Map<String, String> describe(final String path, final TempBlob blob) {
    try {
      return RDescriptionUtils.extractDescriptionFromArchive(path, blob::get);
    }
    catch (InvalidContentException e) {
      log.debug("Unable to read DESCRIPTION of {} in repository {}: {}", path, getRepository().getName(),
          e.getMessage());
      return null;
    }
  }
}
