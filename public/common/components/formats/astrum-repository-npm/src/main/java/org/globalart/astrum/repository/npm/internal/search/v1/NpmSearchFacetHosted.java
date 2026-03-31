package org.globalart.astrum.repository.npm.internal.search.v1;

import java.io.IOException;
import java.time.OffsetDateTime;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;

import org.globalart.astrum.common.entity.Continuation;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.content.facet.ContentFacet;
import org.globalart.astrum.repository.content.fluent.FluentAsset;
import org.globalart.astrum.repository.npm.internal.NpmAttributes.AssetKind;
import org.globalart.astrum.repository.npm.internal.NpmFormat;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.ContentTypes;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.payloads.StringPayload;

import org.joda.time.DateTime;
import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Value;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;
import org.springframework.context.annotation.Scope;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static java.util.stream.Collectors.toList;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_DESCRIPTION;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_KEYWORDS;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_NAME;
import static org.globalart.astrum.repository.npm.internal.NpmAttributes.P_VERSION;
import static org.globalart.astrum.repository.npm.internal.NpmVersionComparator.versionComparator;

@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class NpmSearchFacetHosted
    extends FacetSupport
    implements NpmSearchFacet
{
  private static final int PAGE_SIZE = 500;

  private final NpmSearchParameterExtractor npmSearchParameterExtractor;

  private final NpmSearchResponseFactory npmSearchResponseFactory;

  private final NpmSearchResponseMapper npmSearchResponseMapper;

  private final int v1SearchMaxResults;

  @Autowired
  public NpmSearchFacetHosted(
      final NpmSearchParameterExtractor npmSearchParameterExtractor,
      final NpmSearchResponseFactory npmSearchResponseFactory,
      final NpmSearchResponseMapper npmSearchResponseMapper,
      @Value("${astrum.npm.v1SearchMaxResults:250}") final int v1SearchMaxResults)
  {
    this.npmSearchParameterExtractor = checkNotNull(npmSearchParameterExtractor);
    this.npmSearchResponseFactory = checkNotNull(npmSearchResponseFactory);
    this.npmSearchResponseMapper = checkNotNull(npmSearchResponseMapper);
    this.v1SearchMaxResults = v1SearchMaxResults;
  }

  @Override
  public Content searchV1(final Parameters parameters) throws IOException {
    String text = npmSearchParameterExtractor.extractText(parameters);
    int size = npmSearchParameterExtractor.extractSize(parameters);
    int from = npmSearchParameterExtractor.extractFrom(parameters);

    NpmSearchResponse response;
    if (text.isEmpty()) {
      response = npmSearchResponseFactory.buildEmptyResponse();
    }
    else {
      response = npmSearchResponseFactory.buildResponseForObjects(
          search(text).stream().skip(from).limit(size).collect(toList()));
    }

    String content = npmSearchResponseMapper.writeString(response);
    return new Content(new StringPayload(content, ContentTypes.APPLICATION_JSON));
  }

  private List<NpmSearchResponseObject> search(final String text) {
    List<String> terms = terms(text);
    Map<String, Hit> newest = new LinkedHashMap<>();
    ContentFacet content = facet(ContentFacet.class);
    String continuationToken = null;
    while (true) {
      Continuation<FluentAsset> page = content.assets().byKind(AssetKind.TARBALL.name())
          .browse(PAGE_SIZE, continuationToken);
      for (FluentAsset asset : page) {
        Map<String, Object> npm = asset.attributes(NpmFormat.NAME).backing();
        Object name = npm.get(P_NAME);
        Object version = npm.get(P_VERSION);
        if (name == null || version == null || !matches(terms, npm)) {
          continue;
        }
        Hit hit = new Hit(name.toString(), version.toString(), npm, asset.lastUpdated());
        newest.merge(hit.name, hit, (a, b) -> versionComparator.compare(a.version, b.version) >= 0 ? a : b);
      }
      if (page.size() < PAGE_SIZE) {
        break;
      }
      continuationToken = page.nextContinuationToken();
    }

    String query = String.join(" ", terms);
    return newest.values().stream()
        .sorted(Comparator.comparingDouble((Hit hit) -> score(query, hit)).reversed()
            .thenComparing(hit -> hit.name))
        .limit(v1SearchMaxResults)
        .map(hit -> npmSearchResponseFactory.buildSearchResponseObject(
            hit.attributes,
            hit.lastUpdated == null ? null : new DateTime(hit.lastUpdated.toInstant().toEpochMilli()),
            score(query, hit)))
        .collect(toList());
  }

  private static List<String> terms(final String text) {
    List<String> terms = new ArrayList<>();
    for (String term : text.toLowerCase(Locale.ROOT).split("\\s+")) {
      int colon = term.indexOf(':');
      String value = colon == -1 ? term : term.substring(colon + 1);
      if (!value.isEmpty()) {
        terms.add(value);
      }
    }
    return terms;
  }

  private static boolean matches(final List<String> terms, final Map<String, Object> npm) {
    String haystack = String.join(" ",
        String.valueOf(npm.get(P_NAME)),
        String.valueOf(npm.getOrDefault(P_DESCRIPTION, "")),
        String.join(" ", NpmSearchResponseFactory.keywords(npm.get(P_KEYWORDS))))
        .toLowerCase(Locale.ROOT);
    return terms.stream().allMatch(haystack::contains);
  }

  private static double score(final String query, final Hit hit) {
    String name = hit.name.toLowerCase(Locale.ROOT);
    if (name.equals(query)) {
      return 1.0;
    }
    if (name.contains(query)) {
      return 0.75;
    }
    return 0.5;
  }

  private record Hit(String name, String version, Map<String, Object> attributes, OffsetDateTime lastUpdated)
  {
  }
}
