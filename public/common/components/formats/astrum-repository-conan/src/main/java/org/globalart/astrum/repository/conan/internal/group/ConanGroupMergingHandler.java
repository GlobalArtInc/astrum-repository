package org.globalart.astrum.repository.conan.internal.group;

import java.io.IOException;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeSet;

import javax.annotation.Nonnull;
import javax.annotation.Nullable;

import org.globalart.astrum.repository.conan.internal.ConanEndpoint;
import org.globalart.astrum.repository.conan.internal.ConanJson;
import org.globalart.astrum.repository.conan.internal.ConanRevision;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;

import org.springframework.stereotype.Component;

@Component
public class ConanGroupMergingHandler
    extends GroupHandler
{
  @Nonnull
  @Override
  protected Response doGet(
      @Nonnull final Context context,
      @Nonnull final DispatchedRepositories dispatched) throws Exception
  {
    GroupFacet groupFacet = context.getRepository().facet(GroupFacet.class);
    List<Map<String, Object>> documents = new ArrayList<>();
    Object reference = null;
    for (Response response : getAll(context, groupFacet.members(), dispatched).values()) {
      Map<String, Object> document = read(response);
      if (document != null) {
        documents.add(document);
        if (reference == null) {
          reference = document.get("reference");
        }
      }
    }
    if (documents.isEmpty()) {
      return notFoundResponse(context);
    }
    ConanEndpoint endpoint = ConanEndpoint.of(context);
    return switch (endpoint) {
      case SEARCH_V1, SEARCH_V2 -> ConanJson.ok(ConanJson.results(mergeResults(documents)));
      case RECIPE_LATEST, PACKAGE_LATEST -> ConanJson.ok(newest(documents));
      case RECIPE_REVISIONS, PACKAGE_REVISIONS -> ConanJson.ok(mergeRevisions(documents, reference));
      default -> ConanJson.ok(mergeMaps(documents));
    };
  }

  @Nullable
  private Map<String, Object> read(final Response response) {
    Payload payload = response.getPayload();
    if (!response.getStatus().isSuccessful() || payload == null) {
      return null;
    }
    try (Payload closeable = payload) {
      return ConanJson.read(closeable);
    }
    catch (IOException e) {
      log.debug("Ignoring unreadable Conan member response: {}", e.toString());
      return null;
    }
  }

  private static List<String> mergeResults(final List<Map<String, Object>> documents) {
    TreeSet<String> results = new TreeSet<>(String.CASE_INSENSITIVE_ORDER);
    for (Map<String, Object> document : documents) {
      if (document.get("results") instanceof List<?> list) {
        list.forEach(item -> results.add(String.valueOf(item)));
      }
    }
    return new ArrayList<>(results);
  }

  private static Map<String, Object> newest(final List<Map<String, Object>> documents) {
    ConanRevision newest = null;
    Map<String, Object> chosen = documents.get(0);
    for (Map<String, Object> document : documents) {
      ConanRevision revision = ConanRevision.fromJson(document);
      if (revision != null && (newest == null || revision.time().isAfter(newest.time()))) {
        newest = revision;
        chosen = document;
      }
    }
    return chosen;
  }

  private static Map<String, Object> mergeRevisions(final List<Map<String, Object>> documents,
                                                    @Nullable final Object reference)
  {
    Map<String, ConanRevision> revisions = new LinkedHashMap<>();
    Map<String, Object> raw = new LinkedHashMap<>();
    for (Map<String, Object> document : documents) {
      if (document.get("revisions") instanceof List<?> list) {
        for (Object item : list) {
          ConanRevision revision = ConanRevision.fromJson(item);
          if (revision != null && !revisions.containsKey(revision.revision())) {
            revisions.put(revision.revision(), revision);
            raw.put(revision.revision(), item);
          }
        }
      }
    }
    List<Object> sorted = revisions.values().stream()
        .sorted(ConanRevision.NEWEST_FIRST)
        .map(revision -> raw.get(revision.revision()))
        .toList();
    Map<String, Object> merged = new LinkedHashMap<>();
    if (reference != null) {
      merged.put("reference", reference);
    }
    merged.put("revisions", sorted);
    return merged;
  }

  private static Map<String, Object> mergeMaps(final List<Map<String, Object>> documents) {
    Map<String, Object> merged = new LinkedHashMap<>();
    for (Map<String, Object> document : documents) {
      document.forEach(merged::putIfAbsent);
    }
    return merged;
  }
}
