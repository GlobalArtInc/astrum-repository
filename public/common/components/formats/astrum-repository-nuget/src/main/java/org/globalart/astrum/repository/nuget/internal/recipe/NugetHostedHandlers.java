package org.globalart.astrum.repository.nuget.internal.recipe;

import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.nuget.internal.NugetPaths;
import org.globalart.astrum.repository.nuget.internal.content.NugetContentFacet;
import org.globalart.astrum.repository.nuget.internal.metadata.InvalidNugetPackageException;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetPackageMetadata;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetSearch;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetV2Feed;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetV3Documents;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetVersion;
import org.globalart.astrum.repository.nuget.internal.metadata.NuspecParser;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.PartPayload;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.view.payloads.TempBlob;

import org.slf4j.Logger;
import org.slf4j.LoggerFactory;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.http.HttpResponses.badRequest;
import static org.globalart.astrum.repository.http.HttpResponses.created;
import static org.globalart.astrum.repository.http.HttpResponses.notFound;
import static org.globalart.astrum.repository.http.HttpResponses.ok;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.FILENAME;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.ID;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetMatchers.VERSION;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.group;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.json;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.repositoryUrl;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.text;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.tokens;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.unquote;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.xml;

@Component
public class NugetHostedHandlers
{
  private static final Logger log = LoggerFactory.getLogger(NugetHostedHandlers.class);

  private static final Pattern FILTER_ID =
      Pattern.compile("(?:tolower\\(\\s*Id\\s*\\)|Id)\\s+eq\\s+'((?:[^']|'')*)'", Pattern.CASE_INSENSITIVE);

  private static NugetContentFacet content(final Context context) {
    return context.getRepository().facet(NugetContentFacet.class);
  }

  private static Optional<NugetPackageMetadata> find(final List<NugetPackageMetadata> packages, final String version) {
    Optional<NugetVersion> requested = NugetVersion.parse(version);
    if (requested.isEmpty()) {
      return Optional.empty();
    }
    return packages.stream().filter(metadata -> metadata.getVersion().equals(requested.get())).findFirst();
  }

  public final Handler serviceIndex = context ->
      json(NugetV3Documents.serviceIndex(repositoryUrl(context), true, true));

  public final Handler versions = context -> {
    List<NugetPackageMetadata> packages = content(context).findPackages(tokens(context).get(ID));
    if (packages.isEmpty()) {
      return notFound();
    }
    return json(NugetV3Documents.versions(packages.stream().map(NugetPackageMetadata::getVersion).toList()));
  };

  public final Handler packageContent = context -> {
    Map<String, String> tokens = tokens(context);
    String id = tokens.get(ID);
    Optional<String> version = NugetVersion.normalize(tokens.get(VERSION));
    if (version.isEmpty()) {
      return notFound();
    }
    String filename = tokens.get(FILENAME);
    if (filename != null && !filename.equalsIgnoreCase(NugetPaths.packageFileName(id, version.get()))) {
      return notFound();
    }
    return content(context).get(NugetPaths.packagePath(id, version.get()))
        .map(HttpResponses::ok)
        .orElseGet(HttpResponses::notFound);
  };

  public final Handler nuspec = context -> {
    Map<String, String> tokens = tokens(context);
    String id = tokens.get(ID);
    if (!tokens.get(FILENAME).equalsIgnoreCase(NugetPaths.nuspecFileName(id))) {
      return notFound();
    }
    return content(context).nuspec(id, tokens.get(VERSION))
        .map(bytes -> ok(new BytesPayload(bytes, "application/xml")))
        .orElseGet(HttpResponses::notFound);
  };

  public final Handler registrationIndex = context -> {
    String id = tokens(context).get(ID);
    List<NugetPackageMetadata> packages = content(context).findPackages(id);
    if (packages.isEmpty()) {
      return notFound();
    }
    return json(NugetV3Documents.registrationIndex(repositoryUrl(context), packages.get(0).getId(), packages));
  };

  public final Handler registrationLeaf = context -> {
    Map<String, String> tokens = tokens(context);
    return find(content(context).findPackages(tokens.get(ID)), tokens.get(VERSION))
        .map(metadata -> json(NugetV3Documents.registrationLeaf(repositoryUrl(context), metadata)))
        .orElseGet(HttpResponses::notFound);
  };

  public final Handler catalogEntry = context -> {
    Map<String, String> tokens = tokens(context);
    return find(content(context).findPackages(tokens.get(ID)), tokens.get(VERSION))
        .map(metadata -> json(NugetV3Documents.catalogEntry(repositoryUrl(context), metadata)))
        .orElseGet(HttpResponses::notFound);
  };

  public final Handler search = context -> {
    NugetSearch search = NugetSearch.fromV3(context.getRequest().getParameters());
    NugetSearch.Result result = search.search(content(context).allPackages());
    String url = repositoryUrl(context);
    List<Map<String, Object>> data = new ArrayList<>();
    for (NugetSearch.Hit hit : result.hits()) {
      data.add(NugetV3Documents.searchHit(url, hit.latest(), hit.versions()));
    }
    return json(NugetV3Documents.searchResult(result.totalHits(), data));
  };

  public final Handler autocomplete = context -> {
    Parameters parameters = context.getRequest().getParameters();
    boolean prerelease = Boolean.parseBoolean(parameters.get("prerelease"));
    boolean semVer2 = NugetSearch.isSemVer2(parameters.get("semVerLevel"));
    String id = parameters.get("id");
    if (id != null) {
      NugetSearch filter = new NugetSearch(null, 0, NugetSearch.MAX_TAKE, prerelease, semVer2);
      List<String> versions = content(context).findPackages(id).stream()
          .filter(filter::accepts)
          .sorted(Comparator.comparing(NugetPackageMetadata::getVersion))
          .map(metadata -> metadata.getVersion().toFullString())
          .toList();
      return json(NugetV3Documents.autocompleteResult(versions.size(), versions));
    }
    NugetSearch search = new NugetSearch(parameters.get("q"),
        NugetSearch.intParameter(parameters.get("skip"), 0),
        NugetSearch.intParameter(parameters.get("take"), NugetSearch.DEFAULT_TAKE), prerelease, semVer2);
    NugetSearch.Result result = search.search(content(context).allPackages());
    return json(NugetV3Documents.autocompleteResult(result.totalHits(),
        result.hits().stream().map(hit -> hit.latest().getId()).toList()));
  };

  public final Handler push = context -> {
    Request request = context.getRequest();
    NugetContentFacet content = content(context);
    if (request.isMultipart() && request.getMultiparts() != null) {
      for (PartPayload part : request.getMultiparts()) {
        if (!part.isFormField()) {
          return store(content, part);
        }
      }
      return badRequest("The request does not contain a package");
    }
    Payload payload = request.getPayload();
    if (payload == null || payload.getSize() == 0) {
      return badRequest("The request does not contain a package");
    }
    return store(content, payload);
  };

  private static Response store(final NugetContentFacet content, final Payload payload) throws IOException {
    try (TempBlob blob = content.ingest(payload)) {
      NugetPackageMetadata metadata;
      try (InputStream in = blob.get()) {
        metadata = NuspecParser.parsePackage(in);
      }
      catch (InvalidNugetPackageException | IOException e) {
        log.debug("Rejected invalid NuGet package", e);
        return badRequest("Invalid NuGet package: " + e.getMessage());
      }
      content.putPackage(blob, metadata, null);
      return created();
    }
  }

  public final Handler delete = context -> {
    Map<String, String> tokens = tokens(context);
    return content(context).deletePackage(tokens.get(ID), tokens.get(VERSION))
        ? NugetResponses.noContent()
        : notFound();
  };

  public final Handler v2Root = context -> xml(NugetV2Feed.serviceDocument(repositoryUrl(context)), NugetV2Feed.XML);

  public final Handler v2Metadata = context -> xml(NugetV2Feed.metadataDocument(), NugetV2Feed.XML);

  public final Handler v2FindPackagesById = context -> {
    Parameters parameters = context.getRequest().getParameters();
    String id = unquote(parameters.get("id"));
    if (id == null || id.isEmpty()) {
      return badRequest("Missing id parameter");
    }
    boolean semVer2 = NugetSearch.isSemVer2(parameters.get("semVerLevel"));
    List<NugetPackageMetadata> all = content(context).findPackages(id);
    List<NugetPackageMetadata> entries = NugetV3Documents.sorted(all).stream()
        .filter(metadata -> semVer2 || !metadata.isSemVer2())
        .toList();
    return v2Feed(context, "FindPackagesById", entries, all);
  };

  public final Handler v2PackagesEntry = context -> {
    String id = group(context, 1);
    List<NugetPackageMetadata> all = content(context).findPackages(id);
    return find(all, group(context, 2))
        .map(metadata -> xml(NugetV2Feed.entry(repositoryUrl(context), metadata, all), NugetV2Feed.ATOM_ENTRY))
        .orElseGet(HttpResponses::notFound);
  };

  public final Handler v2Packages = context -> {
    Parameters parameters = context.getRequest().getParameters();
    String filter = parameters.get("$filter");
    List<NugetPackageMetadata> all;
    String id = null;
    if (filter != null) {
      Matcher matcher = FILTER_ID.matcher(filter);
      if (matcher.find()) {
        id = matcher.group(1).replace("''", "'");
      }
    }
    all = id != null ? content(context).findPackages(id) : content(context).allPackages();
    boolean semVer2 = NugetSearch.isSemVer2(parameters.get("semVerLevel"));
    List<NugetPackageMetadata> entries = new ArrayList<>(NugetV3Documents.sorted(all).stream()
        .filter(metadata -> semVer2 || !metadata.isSemVer2())
        .toList());
    if (filter != null && filter.contains("IsAbsoluteLatestVersion")) {
      entries = latestOnly(entries, true);
    }
    else if (filter != null && filter.contains("IsLatestVersion")) {
      entries = latestOnly(entries, false);
    }
    return v2Feed(context, "Packages", page(entries, parameters), all);
  };

  public final Handler v2Search = context -> {
    Parameters parameters = context.getRequest().getParameters();
    String term = unquote(parameters.get("searchTerm"));
    boolean prerelease = Boolean.parseBoolean(unquote(parameters.get("includePrerelease")));
    boolean semVer2 = NugetSearch.isSemVer2(parameters.get("semVerLevel"));
    NugetSearch search = new NugetSearch(term, NugetSearch.intParameter(parameters.get("$skip"), 0),
        NugetSearch.intParameter(parameters.get("$top"), 30), prerelease, semVer2);
    List<NugetPackageMetadata> all = content(context).allPackages();
    NugetSearch.Result result = search.search(all);
    List<NugetPackageMetadata> entries = result.hits().stream().map(NugetSearch.Hit::latest).toList();
    if (context.getRequest().getPath().endsWith("$count")) {
      return text(String.valueOf(result.totalHits()));
    }
    return xml(NugetV2Feed.feed(repositoryUrl(context), "Search", "Search", entries, all), NugetV2Feed.ATOM);
  };

  private static List<NugetPackageMetadata> latestOnly(
      final List<NugetPackageMetadata> entries,
      final boolean includePrerelease)
  {
    Map<String, NugetPackageMetadata> latest = new java.util.LinkedHashMap<>();
    for (NugetPackageMetadata metadata : entries) {
      if (!metadata.isListed() || (!includePrerelease && metadata.getVersion().isPrerelease())) {
        continue;
      }
      latest.merge(metadata.getLowerId(), metadata,
          (left, right) -> left.getVersion().compareTo(right.getVersion()) >= 0 ? left : right);
    }
    return new ArrayList<>(latest.values());
  }

  private static List<NugetPackageMetadata> page(final List<NugetPackageMetadata> entries, final Parameters parameters) {
    int skip = Math.max(0, NugetSearch.intParameter(parameters.get("$skip"), 0));
    int top = Math.max(0, NugetSearch.intParameter(parameters.get("$top"), entries.size()));
    int from = Math.min(skip, entries.size());
    return entries.subList(from, Math.min(entries.size(), from + top));
  }

  private static Response v2Feed(
      final Context context,
      final String title,
      final List<NugetPackageMetadata> entries,
      final List<NugetPackageMetadata> all)
  {
    if (context.getRequest().getPath().endsWith("$count")) {
      return text(String.valueOf(entries.size()));
    }
    return xml(NugetV2Feed.feed(repositoryUrl(context), title, title, entries, all), NugetV2Feed.ATOM);
  }

  public final Handler v2Download = context -> {
    Map<String, String> tokens = tokens(context);
    Optional<String> version = NugetVersion.normalize(tokens.get(VERSION));
    if (version.isEmpty()) {
      return notFound();
    }
    Optional<Content> content = content(context).get(NugetPaths.packagePath(tokens.get(ID), version.get()));
    return content.map(HttpResponses::ok).orElseGet(HttpResponses::notFound);
  };
}
