
package org.globalart.astrum.repository.helm.internal.metadata;

import org.slf4j.LoggerFactory;
import org.slf4j.Logger;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.io.OutputStream;
import java.io.OutputStreamWriter;
import java.io.Reader;
import java.io.UncheckedIOException;
import java.io.Writer;
import java.net.URI;
import java.net.URISyntaxException;
import java.util.Collection;
import java.util.List;
import java.util.Optional;
import java.util.stream.Collectors;

import org.globalart.astrum.repository.view.Content;

import com.fasterxml.jackson.annotation.JsonInclude.Include;
import com.fasterxml.jackson.databind.DeserializationFeature;
import com.fasterxml.jackson.databind.ObjectMapper;
import com.fasterxml.jackson.databind.SerializationFeature;
import com.fasterxml.jackson.dataformat.yaml.YAMLFactory;
import org.globalart.astrum.repository.helm.internal.util.HelmYaml;
import com.fasterxml.jackson.dataformat.yaml.YAMLGenerator;
import com.fasterxml.jackson.dataformat.yaml.YAMLGenerator.Feature;
import org.apache.commons.io.FilenameUtils;
import org.apache.commons.lang3.StringUtils;
import org.apache.http.client.utils.URIBuilder;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Removes absolute URL entries from index.yaml
 *
 * @since 3.28
 */
public class IndexYamlAbsoluteUrlRewriterSupport
{
  protected final Logger log = LoggerFactory.getLogger(getClass());

  private final ObjectMapper objectMapper = new ObjectMapper(HelmYaml.factory()
      .disable(Feature.WRITE_DOC_START_MARKER)
      .configure(YAMLGenerator.Feature.MINIMIZE_QUOTES, true)
      .configure(Feature.ALWAYS_QUOTE_NUMBERS_AS_STRINGS, true))
      .configure(DeserializationFeature.FAIL_ON_UNKNOWN_PROPERTIES, false)
      .setSerializationInclusion(Include.NON_NULL)
      .configure(SerializationFeature.WRITE_DATES_AS_TIMESTAMPS, false);

  protected void updateUrls(
      final InputStream is,
      final OutputStream os)
  {
    try (Reader reader = new InputStreamReader(is);
         Writer writer = new OutputStreamWriter(os)) {
      ChartIndex chartIndex = objectMapper.readValue(reader, ChartIndex.class);
      chartIndex.getEntries().values().stream()
          .flatMap(Collection::stream)
          .forEach(this::rewriteUrls);
      writer.write(objectMapper.writeValueAsString(chartIndex));
    }
    catch (IOException ex) {
      log.error("Error rewriting urls in index.yaml", ex);
    }
  }

  private void rewriteUrls(final ChartEntry chartEntry) {
    List<String> urls = chartEntry.getUrls().stream()
        .map(oldUrl -> rewriteUrl(oldUrl, chartEntry.getName(), chartEntry.getVersion()))
        .filter(Optional::isPresent)
        .map(Optional::get)
        .collect(Collectors.toList());
    chartEntry.setUrls(urls);
  }

  private Optional<String> rewriteUrl(final String oldUrl, final String name, final String version) {
    try {
      URI uri = new URIBuilder(oldUrl).build();
      String fileExtension = FilenameUtils.getExtension(uri.getPath());
      return Optional.of(StringUtils.isNoneBlank(name, version, fileExtension)
          ? name + "-" + version + "." + fileExtension
          : uri.getPath());
    }
    catch (URISyntaxException ex) {
      log.error("Invalid URI in index.yaml", ex);
      return Optional.empty();
    }
  }

  public Optional<String> getFirstUrl(final Content indexYaml, final String filename) {
    checkNotNull(filename);

    try (InputStream inputStream = indexYaml.openInputStream()) {
      ChartIndex chartIndex = objectMapper.readValue(inputStream, ChartIndex.class);
      return chartIndex.getEntries().values().stream()
          .flatMap(Collection::stream)
          .filter(chart -> filename.startsWith(chart.getName() + "-"))
          .filter(chart -> getChartVersion(filename, chart.getName()).equals(chart.getVersion()))
          .flatMap(chart -> chart.getUrls().stream())
          .findFirst();
    }
    catch (IOException e) {
      log.error("Error reading index.yaml");
      throw new UncheckedIOException(e);
    }
  }

  private String getChartVersion(final String filename, final String chartName) {
    return FilenameUtils.removeExtension(filename).replace(chartName + "-", "");
  }
}
