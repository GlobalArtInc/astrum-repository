
package org.globalart.astrum.repository.helm.internal.content.createindex;

import org.slf4j.LoggerFactory;
import org.slf4j.Logger;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import java.util.ArrayList;
import java.util.List;
import java.util.Map;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.content.Asset;
import org.globalart.astrum.repository.content.AssetBlob;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.payloads.StringPayload;
import org.globalart.astrum.repository.helm.internal.AssetKind;
import org.globalart.astrum.repository.helm.internal.HelmFormat;
import org.globalart.astrum.repository.helm.internal.content.HelmContentFacet;
import org.globalart.astrum.repository.helm.internal.metadata.ChartEntry;
import org.globalart.astrum.repository.helm.internal.metadata.ChartIndex;
import org.globalart.astrum.repository.helm.internal.util.YamlParser;

import org.joda.time.DateTime;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.helm.internal.database.HelmProperties.APP_VERSION;
import static org.globalart.astrum.repository.helm.internal.database.HelmProperties.DESCRIPTION;
import static org.globalart.astrum.repository.helm.internal.database.HelmProperties.ICON;
import static org.globalart.astrum.repository.helm.internal.database.HelmProperties.MAINTAINERS;
import static org.globalart.astrum.repository.helm.internal.database.HelmProperties.NAME;
import static org.globalart.astrum.repository.helm.internal.database.HelmProperties.SOURCES;
import static org.globalart.astrum.repository.helm.internal.database.HelmProperties.VERSION;

/**
 * Build index.yaml file for Helm Hosted
 *
 * @since 3.28
 */
@Component
public class CreateIndexServiceImpl
    implements CreateIndexService
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private static final String API_VERSION = "v1";

  private static final String INDEX_YAML_CONTENT_TYPE = "text/x-yaml";

  private final YamlParser yamlParser;

  @Autowired
  public CreateIndexServiceImpl(final YamlParser yamlParser) {
    this.yamlParser = checkNotNull(yamlParser);
  }

  @Nullable
  public Content buildIndexYaml(final Repository repository) {
    HelmContentFacet helmFacet = repository.facet(HelmContentFacet.class);

    ChartIndex index = new ChartIndex();

    for (Asset asset : helmFacet.browseAssets()) {
      if (AssetKind.HELM_PACKAGE.toString().equals(asset.kind())) {
        parseAssetIntoChartEntry(index, asset);
      }
    }

    index.setApiVersion(API_VERSION);
    index.setGenerated(new DateTime());

    return new Content(new StringPayload(yamlParser.getYamlContent(index), INDEX_YAML_CONTENT_TYPE));
  }

  private void parseAssetIntoChartEntry(final ChartIndex index, final Asset asset) {
    NestedAttributesMap formatAttributes = asset.attributes().child(HelmFormat.NAME);
    if (formatAttributes != null) {
      ChartEntry chartEntry = new ChartEntry();
      chartEntry.setName(formatAttributes.get(NAME.getPropertyName(), String.class));
      chartEntry.setVersion(formatAttributes.get(VERSION.getPropertyName(), String.class));
      chartEntry.setDescription(formatAttributes.get(DESCRIPTION.getPropertyName(), String.class));
      chartEntry.setIcon(formatAttributes.get(ICON.getPropertyName(), String.class));
      chartEntry.setCreated(new DateTime(asset.created().toString()));

      @SuppressWarnings("unchecked")
      List<Map<String, String>> maintainers = formatAttributes.get(MAINTAINERS.getPropertyName(), List.class);
      chartEntry.setMaintainers(maintainers);
      chartEntry.setAppVersion(formatAttributes.get(APP_VERSION.getPropertyName(), String.class));
      AssetBlob blob = asset.blob().orElse(null);
      chartEntry.setDigest(blob != null ? blob.checksums().get("sha256") : null);
      createListOfRelativeUrls(formatAttributes, chartEntry);
      chartEntry.setSources(formatAttributes.get(SOURCES.getPropertyName(), List.class));
      index.addEntry(chartEntry);
    }
  }

  private void createListOfRelativeUrls(final NestedAttributesMap formatAttributes, final ChartEntry chartEntry) {
    List<String> urls = new ArrayList<>();
    urls.add(String.format("%s-%s.tgz",
        formatAttributes.get(NAME.getPropertyName(), String.class),
        formatAttributes.get(VERSION.getPropertyName(), String.class)));
    chartEntry.setUrls(urls);
  }
}
