package org.globalart.astrum.repository.cargo.internal;

import java.io.InputStream;
import java.time.Instant;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Objects;
import java.util.Optional;

import org.globalart.astrum.common.hash.HashAlgorithm;
import org.globalart.astrum.repository.cargo.AssetKind;
import org.globalart.astrum.repository.cargo.CargoFormat;
import org.globalart.astrum.repository.cargo.CrateCoordinates;
import org.globalart.astrum.repository.content.Component;
import org.globalart.astrum.repository.content.facet.ContentFacetSupport;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.content.store.FormatStoreManager;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.TempBlob;
import org.globalart.astrum.security.ClientInfo;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.common.entity.Continuations.streamOf;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA1;
import static org.globalart.astrum.common.hash.HashAlgorithm.SHA256;
import static org.globalart.astrum.repository.content.AttributeOperation.SET;
import static org.globalart.astrum.repository.view.ContentTypes.TEXT_PLAIN;

@org.springframework.stereotype.Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
@Qualifier(CargoFormat.NAME)
public class CargoContentFacetImpl
    extends ContentFacetSupport
    implements CargoContentFacet
{
  private static final List<HashAlgorithm> HASH_ALGORITHMS = List.of(SHA1, SHA256);

  private static final String CRATE_CONTENT_TYPE = "application/x-tar";

  private static final String LOWER_NAME_FILTER = "LOWER(name) = #{filterParams.lowerName}";

  private static final String NAME_LIKE_FILTER = "LOWER(name) LIKE #{filterParams.query}";

  @Autowired
  public CargoContentFacetImpl(@Qualifier(CargoFormat.NAME) final FormatStoreManager formatStoreManager) {
    super(formatStoreManager);
  }

  @Override
  public Optional<FluentAsset> getAsset(final String path) {
    return assets().path(checkNotNull(path)).find();
  }

  @Override
  public Optional<FluentAsset> getCrate(final String name, final String version) {
    if (!CrateCoordinates.isValidName(name) || !CrateCoordinates.isValidVersion(version)) {
      return Optional.empty();
    }
    Optional<FluentAsset> exact = getAsset(new CrateCoordinates(name, version).getAssetPath());
    if (exact.isPresent()) {
      return exact;
    }
    return versions(name.toLowerCase(Locale.ROOT)).stream()
        .filter(component -> version.equals(component.version()))
        .flatMap(component -> component.assets().stream())
        .filter(asset -> AssetKind.CRATE.name().equals(asset.kind()))
        .findFirst();
  }

  @Override
  public List<FluentComponent> versions(final String lowerName) {
    return streamOf(components().byFilter(LOWER_NAME_FILTER, Map.of("lowerName", lowerName))::browse)
        .sorted(Comparator.comparing(Component::created))
        .toList();
  }

  @Override
  public Optional<Content> index(final String lowerName) {
    List<String> lines = new ArrayList<>();
    for (FluentComponent component : versions(lowerName)) {
      Map<String, Object> attributes = cargoAttributes(component);
      Object entry = attributes.get(INDEX_ENTRY);
      if (entry instanceof String line) {
        lines.add(CargoIndex.withYanked(line, Boolean.TRUE.equals(attributes.get(YANKED))));
      }
    }
    if (lines.isEmpty()) {
      return Optional.empty();
    }
    return Optional.of(CargoResponses.content(CargoIndex.join(lines), TEXT_PLAIN));
  }

  @Override
  public TempBlob ingest(final InputStream in) {
    return blobs().ingest(in, CRATE_CONTENT_TYPE, HASH_ALGORITHMS);
  }

  @Override
  public FluentAsset publish(final CrateCoordinates coordinates, final JsonNode metadata, final TempBlob crate) {
    for (FluentComponent existing : versions(coordinates.getLowerName())) {
      if (!existing.name().equals(coordinates.getName())) {
        throw CargoException.badRequest(String.format("crate `%s` conflicts with existing crate `%s`",
            coordinates.getName(), existing.name()));
      }
    }
    String cksum = crate.getHashes().get(SHA256).toString();
    ObjectNode entry = CargoIndex.entry(metadata, cksum, Instant.now());
    FluentComponent component = components()
        .name(coordinates.getName())
        .version(coordinates.getVersion())
        .getOrCreate();
    FluentAsset asset = assets()
        .path(coordinates.getAssetPath())
        .kind(AssetKind.CRATE.name())
        .component(component)
        .blob(crate)
        .save();
    Map<String, Object> attributes = new LinkedHashMap<>();
    attributes.put(INDEX_ENTRY, entry.toString());
    attributes.put(YANKED, false);
    clientInfo().map(ClientInfo::getUserid).filter(Objects::nonNull).ifPresent(user -> attributes.put("publisher", user));
    for (String field : List.of("description", "license", "license_file", "homepage", "documentation",
        "repository", "readme_file")) {
      String value = CargoIndex.text(metadata, field);
      if (value != null) {
        attributes.put(field, value);
      }
    }
    for (String field : List.of("authors", "keywords", "categories")) {
      List<String> values = strings(metadata.get(field));
      if (!values.isEmpty()) {
        attributes.put(field, values);
      }
    }
    component.attributes(SET, ATTRIBUTES, attributes);
    return asset;
  }

  @Override
  public void yank(final String name, final String version, final boolean yanked) {
    FluentComponent component = versions(name.toLowerCase(Locale.ROOT)).stream()
        .filter(candidate -> version.equals(candidate.version()))
        .filter(candidate -> cargoAttributes(candidate).get(INDEX_ENTRY) != null)
        .findFirst()
        .orElseThrow(() -> CargoException.notFound(String.format("crate `%s` does not have a version `%s`",
            name, version)));
    Map<String, Object> attributes = new LinkedHashMap<>(cargoAttributes(component));
    attributes.put(YANKED, yanked);
    component.attributes(SET, ATTRIBUTES, attributes);
  }

  @Override
  public Optional<String> publisher(final String lowerName) {
    return versions(lowerName).stream()
        .map(component -> cargoAttributes(component).get("publisher"))
        .filter(String.class::isInstance)
        .map(String.class::cast)
        .findFirst();
  }

  @Override
  public ObjectNode search(final String query, final int perPage) {
    String term = query == null ? "" : query.strip().toLowerCase(Locale.ROOT);
    Map<String, List<FluentComponent>> byName = new LinkedHashMap<>();
    streamOf(components().byFilter(NAME_LIKE_FILTER, Map.of("query", "%" + escapeLike(term) + "%"))::browse)
        .filter(component -> cargoAttributes(component).get(INDEX_ENTRY) != null)
        .forEach(component -> byName.computeIfAbsent(component.name(), key -> new ArrayList<>()).add(component));

    List<Map.Entry<String, FluentComponent>> matches = new ArrayList<>();
    byName.forEach((name, components) -> latest(components).ifPresent(latest -> matches.add(Map.entry(name, latest))));
    matches.sort(Comparator
        .comparing((Map.Entry<String, FluentComponent> match) -> !match.getKey().equalsIgnoreCase(term))
        .thenComparing(match -> match.getKey().toLowerCase(Locale.ROOT)));

    ObjectNode result = CargoResponses.object();
    ArrayNode crates = result.putArray("crates");
    matches.stream().limit(Math.max(perPage, 0)).forEach(match -> {
      ObjectNode crate = crates.addObject();
      crate.put("name", match.getKey());
      crate.put("max_version", match.getValue().version());
      Object description = cargoAttributes(match.getValue()).get("description");
      crate.put("description", description instanceof String text ? text : "");
    });
    result.putObject("meta").put("total", matches.size());
    return result;
  }

  private static Optional<FluentComponent> latest(final List<FluentComponent> components) {
    Comparator<FluentComponent> byVersion = Comparator.comparing(
        component -> Objects.toString(component.normalizedVersion(), component.version()));
    Optional<FluentComponent> latest = components.stream()
        .filter(component -> !Boolean.TRUE.equals(cargoAttributes(component).get(YANKED)))
        .max(byVersion);
    return latest.isPresent() ? latest : components.stream().max(byVersion);
  }

  @Override
  public FluentAsset saveIndex(final String path, final Content content) {
    try (TempBlob blob = blobs().ingest(content, HASH_ALGORITHMS)) {
      return assets()
          .path(path)
          .kind(AssetKind.INDEX.name())
          .blob(blob)
          .save();
    }
  }

  @Override
  public FluentAsset saveCrate(final CrateCoordinates coordinates, final Content content) {
    FluentComponent component = components()
        .name(coordinates.getName())
        .version(coordinates.getVersion())
        .getOrCreate();
    try (TempBlob blob = blobs().ingest(content, HASH_ALGORITHMS)) {
      return assets()
          .path(coordinates.getAssetPath())
          .kind(AssetKind.CRATE.name())
          .component(component)
          .blob(blob)
          .save();
    }
  }

  @SuppressWarnings("unchecked")
  private static Map<String, Object> cargoAttributes(final Component component) {
    Object attributes = component.attributes().get(ATTRIBUTES);
    return attributes instanceof Map<?, ?> map ? (Map<String, Object>) map : new HashMap<>();
  }

  private static List<String> strings(final JsonNode node) {
    List<String> values = new ArrayList<>();
    if (node != null && node.isArray()) {
      node.forEach(value -> {
        if (value.isTextual()) {
          values.add(value.asText());
        }
      });
    }
    return values;
  }

  private static String escapeLike(final String term) {
    return term.replace("\\", "\\\\").replace("%", "\\%").replace("_", "\\_");
  }
}
