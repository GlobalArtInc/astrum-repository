package org.globalart.astrum.repository.nuget.internal.group;

import java.io.IOException;
import java.io.InputStream;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Map.Entry;
import java.util.Optional;
import java.util.Set;
import java.util.regex.Matcher;
import java.util.regex.Pattern;

import javax.annotation.Nullable;

import org.globalart.astrum.common.collect.AttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.nuget.internal.NugetFormat;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetSearch;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetV2Feed;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetV3Documents;
import org.globalart.astrum.repository.nuget.internal.metadata.NugetVersion;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Parameters;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;

import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.http.HttpResponses.notFound;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.json;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.repositoryUrl;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.text;
import static org.globalart.astrum.repository.nuget.internal.recipe.NugetResponses.xml;

@Component
@Qualifier(NugetFormat.NAME)
public class NugetGroupHandler
    extends GroupHandler
{
  private static final Pattern ENTRY = Pattern.compile("<entry[\\s>].*?</entry>", Pattern.DOTALL);

  private static final Pattern ENTRY_ID = Pattern.compile("<d:Id>([^<]*)</d:Id>");

  private static final Pattern ENTRY_VERSION = Pattern.compile("<d:Version>([^<]*)</d:Version>");

  private static final Pattern ROOT_ENTRY = Pattern.compile("<entry[^>]*>");

  private static final Pattern FEED_START = Pattern.compile("<feed[^>]*>");

  private record MemberText(Repository member, String text)
  {
  }

  public final Handler serviceIndex = context ->
      json(NugetV3Documents.serviceIndex(repositoryUrl(context), false, true));

  public final Handler v2Root = context -> xml(NugetV2Feed.serviceDocument(repositoryUrl(context)), NugetV2Feed.XML);

  public final Handler v2Metadata = context -> xml(NugetV2Feed.metadataDocument(), NugetV2Feed.XML);

  public final Handler firstBinary = context -> getFirst(context, members(context), dispatched(context));

  public final Handler firstRewritten = context -> {
    Response response = getFirst(context, members(context), dispatched(context));
    if (response.getStatus().getCode() != HttpStatus.OK || response.getPayload() == null) {
      return response;
    }
    String contentType = response.getPayload().getContentType();
    String body = read(response.getPayload());
    String rewritten = rewrite(context, body);
    return contentType != null && contentType.contains("xml") ? xml(rewritten, contentType) : json(rewritten);
  };

  public final Handler versions = context -> {
    List<MemberText> responses = collect(context, context.getRequest());
    if (responses.isEmpty()) {
      return notFound();
    }
    Set<NugetVersion> versions = new LinkedHashSet<>();
    for (MemberText response : responses) {
      Object list = NugetV3Documents.read(response.text()).get("versions");
      if (list instanceof List<?> items) {
        for (Object item : items) {
          if (item instanceof String string) {
            NugetVersion.parse(string).ifPresent(versions::add);
          }
        }
      }
    }
    return json(NugetV3Documents.versions(versions));
  };

  public final Handler registrationIndex = context -> {
    List<MemberText> responses = collect(context, context.getRequest());
    if (responses.isEmpty()) {
      return notFound();
    }
    if (responses.size() == 1) {
      return json(rewrite(context, responses.get(0).text()));
    }
    Map<NugetVersion, Object> leaves = new LinkedHashMap<>();
    for (MemberText response : responses) {
      Map<String, Object> index = NugetV3Documents.read(response.text());
      if (!(index.get("items") instanceof List<?> pages)) {
        continue;
      }
      for (Object pageObject : pages) {
        if (!(pageObject instanceof Map<?, ?> page)) {
          continue;
        }
        Object items = page.get("items");
        if (items == null && page.get("@id") instanceof String pageUrl) {
          items = fetchPage(context, response.member(), pageUrl).get("items");
        }
        if (items instanceof List<?> leafList) {
          for (Object leaf : leafList) {
            version(leaf).ifPresent(version -> leaves.putIfAbsent(version, leaf));
          }
        }
      }
    }
    if (leaves.isEmpty()) {
      return notFound();
    }
    List<Entry<NugetVersion, Object>> sorted = new ArrayList<>(leaves.entrySet());
    sorted.sort(Entry.comparingByKey());
    String indexUrl = repositoryUrl(context) + context.getRequest().getPath();
    String lower = sorted.get(0).getKey().toLowerString();
    String upper = sorted.get(sorted.size() - 1).getKey().toLowerString();
    Map<String, Object> page = new LinkedHashMap<>();
    page.put("@id", indexUrl + "#page/" + lower + "/" + upper);
    page.put("@type", "catalog:CatalogPage");
    page.put("count", sorted.size());
    page.put("items", sorted.stream().map(Entry::getValue).toList());
    page.put("lower", lower);
    page.put("parent", indexUrl);
    page.put("upper", upper);
    Map<String, Object> index = new LinkedHashMap<>();
    index.put("@id", indexUrl);
    index.put("@type", List.of("catalog:CatalogRoot", "PackageRegistration", "catalog:Permalink"));
    index.put("count", 1);
    index.put("items", List.of(page));
    return json(rewrite(context, NugetV3Documents.write(index)));
  };

  public final Handler search = context -> {
    Parameters parameters = context.getRequest().getParameters();
    int skip = Math.max(0, NugetSearch.intParameter(parameters.get("skip"), 0));
    int take = Math.max(0, Math.min(NugetSearch.MAX_TAKE,
        NugetSearch.intParameter(parameters.get("take"), NugetSearch.DEFAULT_TAKE)));
    Parameters memberParameters = copy(parameters);
    memberParameters.replace("skip", "0");
    memberParameters.replace("take", String.valueOf(Math.min(NugetSearch.MAX_TAKE, skip + take)));
    List<MemberText> responses = collect(context, subRequest(context.getRequest(), null, memberParameters));
    Map<String, Map<String, Object>> merged = new LinkedHashMap<>();
    long totalHits = 0;
    long duplicates = 0;
    for (MemberText response : responses) {
      Map<String, Object> result = NugetV3Documents.read(response.text());
      if (result.get("totalHits") instanceof Number number) {
        totalHits += number.longValue();
      }
      if (!(result.get("data") instanceof List<?> data)) {
        continue;
      }
      for (Object item : data) {
        if (!(item instanceof Map<?, ?> hit) || !(hit.get("id") instanceof String id)) {
          continue;
        }
        @SuppressWarnings("unchecked")
        Map<String, Object> typedHit = (Map<String, Object>) hit;
        Map<String, Object> existing = merged.get(id.toLowerCase(Locale.ROOT));
        if (existing == null) {
          merged.put(id.toLowerCase(Locale.ROOT), new LinkedHashMap<>(typedHit));
        }
        else {
          duplicates++;
          mergeVersions(existing, typedHit);
        }
      }
    }
    List<Map<String, Object>> hits = new ArrayList<>(merged.values());
    int from = Math.min(skip, hits.size());
    List<Map<String, Object>> page = hits.subList(from, Math.min(hits.size(), from + take));
    return json(rewrite(context, NugetV3Documents.searchResult(Math.max(hits.size(), totalHits - duplicates), page)));
  };

  public final Handler autocomplete = context -> {
    Parameters parameters = context.getRequest().getParameters();
    int skip = Math.max(0, NugetSearch.intParameter(parameters.get("skip"), 0));
    int take = Math.max(0, Math.min(NugetSearch.MAX_TAKE,
        NugetSearch.intParameter(parameters.get("take"), NugetSearch.DEFAULT_TAKE)));
    boolean byId = parameters.get("id") != null;
    Parameters memberParameters = copy(parameters);
    if (!byId) {
      memberParameters.replace("skip", "0");
      memberParameters.replace("take", String.valueOf(Math.min(NugetSearch.MAX_TAKE, skip + take)));
    }
    List<MemberText> responses = collect(context, subRequest(context.getRequest(), null, memberParameters));
    LinkedHashMap<String, String> values = new LinkedHashMap<>();
    for (MemberText response : responses) {
      if (NugetV3Documents.read(response.text()).get("data") instanceof List<?> data) {
        for (Object item : data) {
          if (item instanceof String string) {
            values.putIfAbsent(string.toLowerCase(Locale.ROOT), string);
          }
        }
      }
    }
    List<String> result = new ArrayList<>(values.values());
    if (byId) {
      result.sort(Comparator.comparing(value -> NugetVersion.parse(value).orElse(null),
          Comparator.nullsFirst(Comparator.naturalOrder())));
      return json(NugetV3Documents.autocompleteResult(result.size(), result));
    }
    int from = Math.min(skip, result.size());
    return json(NugetV3Documents.autocompleteResult(result.size(),
        result.subList(from, Math.min(result.size(), from + take))));
  };

  public final Handler v2Feed = context -> {
    List<MemberText> responses = collect(context, context.getRequest());
    if (responses.isEmpty()) {
      return notFound();
    }
    if (context.getRequest().getPath().endsWith("$count")) {
      long count = 0;
      for (MemberText response : responses) {
        count += NugetSearch.intParameter(response.text().trim(), 0);
      }
      return text(String.valueOf(count));
    }
    if (responses.size() == 1) {
      return xml(rewrite(context, responses.get(0).text()), NugetV2Feed.ATOM);
    }
    Map<String, String> entries = new LinkedHashMap<>();
    for (MemberText response : responses) {
      Matcher matcher = ENTRY.matcher(response.text());
      while (matcher.find()) {
        String entry = ROOT_ENTRY.matcher(matcher.group()).replaceFirst("<entry>");
        entries.putIfAbsent(entryKey(entry), entry);
      }
    }
    String first = responses.get(0).text();
    Matcher feedStart = FEED_START.matcher(first);
    String header = feedStart.find() ? first.substring(0, feedStart.end()) : "<feed>";
    StringBuilder builder = new StringBuilder(header);
    builder.append("<m:count>").append(entries.size()).append("</m:count>");
    builder.append("<title type=\"text\">").append(context.getRequest().getPath().substring(1)).append("</title>");
    entries.values().forEach(builder::append);
    builder.append("</feed>");
    return xml(rewrite(context, builder.toString()), NugetV2Feed.ATOM);
  };

  private static String entryKey(final String entry) {
    Matcher id = ENTRY_ID.matcher(entry);
    Matcher version = ENTRY_VERSION.matcher(entry);
    String idValue = id.find() ? id.group(1) : entry;
    String versionValue = version.find() ? version.group(1) : "";
    return idValue.toLowerCase(Locale.ROOT) + "/"
        + NugetVersion.parse(versionValue).map(NugetVersion::toLowerString).orElse(versionValue);
  }

  @SuppressWarnings("unchecked")
  private static void mergeVersions(final Map<String, Object> existing, final Map<String, Object> other) {
    if (!(existing.get("versions") instanceof List<?> existingVersions)
        || !(other.get("versions") instanceof List<?> otherVersions)) {
      return;
    }
    Map<String, Object> byVersion = new LinkedHashMap<>();
    for (Object version : existingVersions) {
      if (version instanceof Map<?, ?> map && map.get("version") instanceof String string) {
        byVersion.putIfAbsent(string.toLowerCase(Locale.ROOT), version);
      }
    }
    for (Object version : otherVersions) {
      if (version instanceof Map<?, ?> map && map.get("version") instanceof String string) {
        byVersion.putIfAbsent(string.toLowerCase(Locale.ROOT), version);
      }
    }
    List<Object> versions = new ArrayList<>(byVersion.values());
    versions.sort(Comparator.comparing(
        version -> NugetVersion.parse((String) ((Map<String, Object>) version).get("version")).orElse(null),
        Comparator.nullsFirst(Comparator.naturalOrder())));
    existing.put("versions", versions);
  }

  private static Optional<NugetVersion> version(final Object leaf) {
    if (leaf instanceof Map<?, ?> map && map.get("catalogEntry") instanceof Map<?, ?> entry
        && entry.get("version") instanceof String version) {
      return NugetVersion.parse(version);
    }
    return Optional.empty();
  }

  private Map<String, Object> fetchPage(final Context context, final Repository member, final String pageUrl)
      throws Exception
  {
    String memberUrl = member.getUrl();
    if (!pageUrl.startsWith(memberUrl + "/")) {
      return Map.of();
    }
    Request request = subRequest(context.getRequest(), pageUrl.substring(memberUrl.length()),
        new Parameters());
    Response response = member.facet(ViewFacet.class).dispatch(request, context);
    if (response.getStatus().getCode() != HttpStatus.OK || response.getPayload() == null) {
      return Map.of();
    }
    return NugetV3Documents.read(read(response.getPayload()));
  }

  private List<MemberText> collect(final Context context, final Request request) throws Exception {
    List<MemberText> result = new ArrayList<>();
    Map<Repository, Response> responses =
        getAll(request, context, members(context), new DispatchedRepositories());
    for (Map.Entry<Repository, Response> entry : responses.entrySet()) {
      Response response = entry.getValue();
      if (response.getStatus().getCode() == HttpStatus.OK && response.getPayload() != null) {
        result.add(new MemberText(entry.getKey(), read(response.getPayload())));
      }
    }
    return result;
  }

  private static List<Repository> members(final Context context) {
    return context.getRepository().facet(GroupFacet.class).members();
  }

  private static DispatchedRepositories dispatched(final Context context) {
    return context.getRequest().getAttributes().getOrCreate(DispatchedRepositories.class);
  }

  private static Request subRequest(
      final Request original,
      @Nullable final String path,
      final Parameters parameters)
  {
    Map<String, Object> attributes = new HashMap<>(original.getAttributes().backing());
    attributes.remove(DispatchedRepositories.class.getName());
    Request.Builder builder = new Request.Builder().copy(original)
        .attributes(new AttributesMap(attributes))
        .parameters(parameters);
    if (path != null) {
      builder.path(path);
    }
    return builder.build();
  }

  private static Parameters copy(final Parameters parameters) {
    Parameters copy = new Parameters();
    for (String name : parameters.names()) {
      copy.set(name, parameters.getAll(name));
    }
    return copy;
  }

  private static String read(final Payload payload) throws IOException {
    try (InputStream in = payload.openInputStream()) {
      return new String(in.readAllBytes(), StandardCharsets.UTF_8);
    }
    finally {
      payload.close();
    }
  }

  private static String rewrite(final Context context, final String text) {
    String groupUrl = context.getRepository().getUrl() + "/";
    String result = text;
    for (Repository member : context.getRepository().facet(GroupFacet.class).members()) {
      result = result.replace(member.getUrl() + "/", groupUrl);
    }
    return result;
  }
}
