package org.globalart.astrum.repository.cargo.internal;

import java.io.InputStream;
import java.util.List;
import java.util.Optional;

import org.globalart.astrum.repository.Facet;
import org.globalart.astrum.repository.cargo.CrateCoordinates;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.content.fluent.FluentComponent;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;

@Facet.Exposed
public interface CargoContentFacet
    extends ContentFacet
{
  String ATTRIBUTES = "cargo";

  String INDEX_ENTRY = "index";

  String YANKED = "yanked";

  Optional<FluentAsset> getAsset(String path);

  Optional<FluentAsset> getCrate(String name, String version);

  List<FluentComponent> versions(String lowerName);

  Optional<Content> index(String lowerName);

  TempBlob ingest(InputStream in);

  FluentAsset publish(CrateCoordinates coordinates, JsonNode metadata, TempBlob crate);

  void yank(String name, String version, boolean yanked);

  Optional<String> publisher(String lowerName);

  ObjectNode search(String query, int perPage);

  FluentAsset saveIndex(String path, Content content);

  FluentAsset saveCrate(CrateCoordinates coordinates, Content content);
}
