
package org.globalart.astrum.repository.conan.internal.content;

import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;
import java.util.TreeMap;

import javax.annotation.Nullable;

import org.globalart.astrum.common.entity.Continuations;
import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.config.WritePolicy;
import org.globalart.astrum.repository.conan.AssetKind;
import org.globalart.astrum.repository.conan.ConanContentFacet;
import org.globalart.astrum.repository.conan.ConanCoords;
import org.globalart.astrum.repository.conan.ConanFormat;
import org.globalart.astrum.repository.conan.internal.ConanInfo;
import org.globalart.astrum.repository.conan.internal.ConanPaths;
import org.globalart.astrum.repository.conan.internal.ConanReferencePattern;
import org.globalart.astrum.repository.conan.internal.ConanRevision;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.fluent.FluentQuery;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.nio.charset.StandardCharsets.UTF_8;
import static org.globalart.astrum.common.hash.HashAlgorithm.MD5;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW;
import static org.globalart.astrum.repository.config.WritePolicy.ALLOW_ONCE;

@org.springframework.stereotype.Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(ConanFormat.NAME)
public class ConanContentFacetImpl
    extends ContentFacetSupport
    implements ConanContentFacet
{
  public static final List<HashAlgorithm> HASH_ALGORITHMS = List.of(SHA1, SHA256, MD5);

  private static final int PAGE = 500;

  private static final String PATH_PREFIX_FILTER = "path LIKE #{filterParams.prefix} ESCAPE '\\'";

  @Autowired
  public ConanContentFacetImpl(@Qualifier(ConanFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  protected WritePolicy writePolicy(final Asset asset) {
    WritePolicy writePolicy = super.writePolicy(asset);
    if (writePolicy == ALLOW_ONCE && (ConanPaths.isMetadata(asset.path()) || !isFileKind(asset.kind()))) {
      return ALLOW;
    }
    return writePolicy;
  }

  private static boolean isFileKind(@Nullable final String kind) {
    if (kind == null) {
      return true;
    }
    try {
      return AssetKind.valueOf(kind).isFile();
    }
    catch (IllegalArgumentException e) {
      return true;
    }
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    return assets().path(checkNotNull(path)).find();
  }

  @Override
  public TempBlob ingest(final Payload payload) {
    return blobs().ingest(checkNotNull(payload), HASH_ALGORITHMS);
  }

  @Override
  public FluentAsset put(final ConanCoords coords, final String path, final AssetKind kind, final Payload payload) {
    try (TempBlob blob = ingest(payload)) {
      return put(coords, path, kind, blob);
    }
  }

  @Override
  public FluentAsset put(final ConanCoords coords, final String path, final AssetKind kind, final TempBlob blob) {
    checkNotNull(coords);
    checkNotNull(path);
    checkNotNull(kind);
    checkNotNull(blob);
    FluentComponent component = component(coords);
    FluentAsset asset = assets()
        .path(path)
        .kind(kind.name())
        .component(component)
        .attributes(ConanFormat.NAME, assetAttributes(coords, kind, blob))
        .blob(blob)
        .save();
    if (kind == AssetKind.RECIPE_MANIFEST && coords.getRecipeRevision() != null) {
      latestRecipeRevision(coords).ifPresent(latest -> updateComponentRevision(component, coords, latest));
    }
    return asset;
  }

  private Map<String, Object> assetAttributes(final ConanCoords coords, final AssetKind kind, final TempBlob blob) {
    Map<String, Object> attributes = new LinkedHashMap<>();
    putIfNotNull(attributes, "recipeRevision", coords.getRecipeRevision());
    putIfNotNull(attributes, "packageId", coords.getPackageId());
    putIfNotNull(attributes, "packageRevision", coords.getPackageRevision());
    if (kind == AssetKind.PACKAGE_FILE && ConanPaths.CONAN_INFO.equals(coords.getFile())) {
      ConanInfo info = ConanInfo.parse(read(blob));
      Map<String, String> settings = info.values("settings");
      if (!settings.isEmpty()) {
        attributes.put("settings", settings);
      }
      Map<String, String> options = info.values("options");
      if (!options.isEmpty()) {
        attributes.put("options", options);
      }
    }
    return attributes;
  }

  private static void putIfNotNull(final Map<String, Object> map, final String key, @Nullable final Object value) {
    if (value != null) {
      map.put(key, value);
    }
  }

  private static String read(final TempBlob blob) {
    try (InputStream in = blob.get()) {
      return new String(in.readAllBytes(), UTF_8);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  private FluentComponent component(final ConanCoords coords) {
    Map<String, Object> attributes = componentAttributes(coords, null);
    return components()
        .name(coords.getName())
        .namespace(coords.namespace())
        .version(coords.getVersion())
        .attributes(ConanFormat.NAME, attributes)
        .getOrCreate();
  }

  private static Map<String, Object> componentAttributes(final ConanCoords coords, @Nullable final String revision) {
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put("baseVersion", coords.getVersion());
    attributes.put("user", coords.getUser());
    attributes.put("channel", coords.getChannel());
    putIfNotNull(attributes, "revision", revision);
    return attributes;
  }

  private void updateComponentRevision(
      final FluentComponent component,
      final ConanCoords coords,
      final ConanRevision latest)
  {
    Object current = component.attributes(ConanFormat.NAME).get("revision");
    if (!Objects.equals(current, latest.revision())) {
      component.withAttribute(ConanFormat.NAME, componentAttributes(coords, latest.revision()));
    }
  }

  @Override
  public List<ConanRevision> recipeRevisions(final ConanCoords coords) {
    String prefix = ConanPaths.recipeBase(coords) + "/";
    return revisions(prefix, AssetKind.RECIPE_MANIFEST, "/" + ConanPaths.EXPORT + "/" + ConanPaths.CONAN_MANIFEST);
  }

  @Override
  public Optional<ConanRevision> latestRecipeRevision(final ConanCoords coords) {
    return recipeRevisions(coords).stream().findFirst();
  }

  @Override
  public boolean recipeRevisionExists(final ConanCoords coords) {
    return getAsset(ConanPaths.recipeFile(coords, ConanPaths.CONAN_MANIFEST)).isPresent();
  }

  @Override
  public List<ConanRevision> packageRevisions(final ConanCoords coords) {
    String prefix = ConanPaths.packageBase(coords) + "/";
    return revisions(prefix, AssetKind.PACKAGE_MANIFEST, "/" + ConanPaths.CONAN_MANIFEST);
  }

  @Override
  public Optional<ConanRevision> latestPackageRevision(final ConanCoords coords) {
    return packageRevisions(coords).stream().findFirst();
  }

  @Override
  public boolean packageRevisionExists(final ConanCoords coords) {
    return getAsset(ConanPaths.packageFile(coords, ConanPaths.CONAN_MANIFEST)).isPresent();
  }

  private List<ConanRevision> revisions(final String prefix, final AssetKind kind, final String suffix) {
    List<ConanRevision> revisions = new ArrayList<>();
    for (FluentAsset asset : assetsUnder(prefix, kind)) {
      String rest = asset.path().substring(prefix.length());
      if (!rest.endsWith(suffix)) {
        continue;
      }
      String revision = rest.substring(0, rest.length() - suffix.length());
      if (revision.isEmpty() || revision.contains("/")) {
        continue;
      }
      revisions.add(new ConanRevision(revision, revisionTime(asset)));
    }
    revisions.sort(ConanRevision.NEWEST_FIRST);
    return revisions;
  }

  private static OffsetDateTime revisionTime(final Asset asset) {
    return asset.blob().map(AssetBlob::blobCreated).filter(Objects::nonNull).orElse(asset.created());
  }

  @Override
  public Map<String, FluentAsset> files(final String directory) {
    Map<String, FluentAsset> files = new TreeMap<>();
    for (FluentAsset asset : assetsUnder(directory, null)) {
      String name = asset.path().substring(directory.length());
      if (!name.isEmpty() && isFileKind(asset.kind())) {
        files.put(name, asset);
      }
    }
    return files;
  }

  @Override
  public Map<String, ConanInfo> packageInfos(final ConanCoords coords) {
    String prefix = ConanPaths.packagesBase(coords);
    Map<String, ConanRevision> latest = new HashMap<>();
    for (FluentAsset asset : assetsUnder(prefix, AssetKind.PACKAGE_MANIFEST)) {
      String[] parts = asset.path().substring(prefix.length()).split("/");
      if (parts.length != 3 || !ConanPaths.CONAN_MANIFEST.equals(parts[2])) {
        continue;
      }
      ConanRevision revision = new ConanRevision(parts[1], revisionTime(asset));
      latest.merge(parts[0], revision,
          (a, b) -> ConanRevision.NEWEST_FIRST.compare(a, b) <= 0 ? a : b);
    }
    Map<String, ConanInfo> infos = new TreeMap<>();
    latest.forEach((packageId, revision) -> {
      String path = prefix + packageId + "/" + revision.revision() + "/" + ConanPaths.CONAN_INFO;
      getAsset(path).ifPresent(asset -> infos.put(packageId, ConanInfo.parse(content(asset))));
    });
    return infos;
  }

  private static String content(final FluentAsset asset) {
    try (InputStream in = asset.download().openInputStream()) {
      return new String(in.readAllBytes(), UTF_8);
    }
    catch (IOException e) {
      throw new UncheckedIOException(e);
    }
  }

  @Override
  public List<ConanCoords> searchRecipes(final ConanReferencePattern pattern) {
    FluentQuery<FluentComponent> query = components().byFilter("LOWER(name) LIKE #{filterParams.name} ESCAPE '\\'",
        Map.of("name", pattern.nameLikePattern()));
    List<ConanCoords> found = new ArrayList<>();
    for (FluentComponent component : Continuations.iterableOf(query::browse, PAGE)) {
      ConanCoords coords = coords(component);
      if (pattern.matches(coords)) {
        found.add(coords);
      }
    }
    found.sort((a, b) -> a.reference().compareToIgnoreCase(b.reference()));
    return found;
  }

  private static ConanCoords coords(final Component component) {
    Map<String, Object> attributes = component.attributes(ConanFormat.NAME).backing();
    String version = attributes.get("baseVersion") instanceof String text ? text : component.version();
    String user = attributes.get("user") instanceof String text ? text : null;
    String channel = attributes.get("channel") instanceof String text ? text : null;
    return new ConanCoords(component.name(), version, user, channel, null, null, null, null);
  }

  @Override
  public int deleteUnder(final ConanCoords recipe, final String prefix) {
    int deleted = 0;
    for (FluentAsset asset : assetsUnder(prefix, null)) {
      if (asset.delete()) {
        deleted++;
      }
    }
    components()
        .name(recipe.getName())
        .namespace(recipe.namespace())
        .version(recipe.getVersion())
        .find()
        .ifPresent(component -> {
          if (component.assets(false).isEmpty()) {
            component.delete();
          }
          else {
            ConanCoords coords = recipe.recipe();
            String revision = latestRecipeRevision(coords).map(ConanRevision::revision).orElse(null);
            if (!Objects.equals(component.attributes(ConanFormat.NAME).get("revision"), revision)) {
              component.withAttribute(ConanFormat.NAME, componentAttributes(coords, revision));
            }
          }
        });
    return deleted;
  }

  private List<FluentAsset> assetsUnder(final String prefix, @Nullable final AssetKind kind) {
    Map<String, Object> params = new HashMap<>();
    params.put("prefix", ConanPaths.escapeLike(prefix) + "%");
    String filter = PATH_PREFIX_FILTER;
    if (kind != null) {
      params.put("kind", kind.name());
      filter = filter + " AND kind = #{filterParams.kind}";
    }
    FluentQuery<FluentAsset> query = assets().byFilter(filter, params);
    List<FluentAsset> result = new ArrayList<>();
    Continuations.iterableOf(query::browse, PAGE).forEach(result::add);
    return result;
  }
}
