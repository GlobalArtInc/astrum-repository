package org.globalart.astrum.repository.terraform.internal.group;

import java.io.IOException;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpMethods;
import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformRequest;
import org.globalart.astrum.repository.terraform.internal.DownloadTokens;
import org.globalart.astrum.repository.terraform.internal.SemanticVersions;
import org.globalart.astrum.repository.terraform.internal.TerraformResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ArrayNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.springframework.stereotype.Component;

/**
 * Answers the version lists of a group with the union of its members' lists. For a version several members have,
 * the first member in group order wins, as it does when the package itself is downloaded.
 */
@Component
public class TerraformGroupMergeHandler
    extends GroupHandler
{
  @Nonnull
  @Override
  protected Response doGet(@Nonnull final Context context, @Nonnull final DispatchedRepositories dispatched)
      throws Exception
  {
    GroupFacet groupFacet = context.getRepository().facet(GroupFacet.class);
    Request request = new Request.Builder().copy(context.getRequest()).action(HttpMethods.GET).build();
    List<JsonNode> documents = new ArrayList<>();
    for (Response response : getAll(request, context, groupFacet.members(), dispatched).values()) {
      byte[] body = TerraformResponses.body(response);
      if (body != null) {
        try {
          documents.add(TerraformResponses.read(body));
        }
        catch (IOException e) {
          log.debug("Skipping an unreadable member answer for {}", context.getRequest().getPath(), e);
        }
      }
    }
    if (documents.isEmpty()) {
      return TerraformResponses.error(404, "Not found in any member of " + context.getRepository().getName());
    }
    return TerraformResponses.json(switch (TerraformRequest.of(context).endpoint()) {
      case MODULE_VERSIONS -> mergeModuleVersions(documents);
      case PROVIDER_VERSIONS -> mergeProviderVersions(documents);
      case MIRROR_INDEX -> mergeMirrorIndexes(documents);
      case MIRROR_VERSION -> resign(mergeMirrorVersions(documents), context);
      default -> throw new IllegalStateException("Not a list endpoint: " + context.getRequest().getPath());
    });
  }

  private static ObjectNode resign(final ObjectNode document, final Context context) {
    DownloadTokens tokens = DownloadTokens.of(context.getRepository().facet(TerraformContentFacet.class));
    String documentPath = context.getRequest().getPath();
    document.path("archives").forEach(archive -> {
      if (archive instanceof ObjectNode entry && entry.hasNonNull("url")) {
        entry.put("url", tokens.resign(entry.get("url").asText(), documentPath));
      }
    });
    return document;
  }

  static ObjectNode mergeModuleVersions(final List<JsonNode> documents) {
    Map<String, JsonNode> versions = new TreeMap<>(SemanticVersions.ORDER);
    for (JsonNode document : documents) {
      JsonNode module = document.path("modules").path(0);
      for (JsonNode version : module.path("versions")) {
        String value = version.path("version").asText();
        if (!value.isEmpty()) {
          versions.putIfAbsent(value, version);
        }
      }
    }
    ObjectNode merged = TerraformResponses.object();
    ArrayNode list = merged.putArray("modules").addObject().putArray("versions");
    versions.values().forEach(list::add);
    return merged;
  }

  static ObjectNode mergeProviderVersions(final List<JsonNode> documents) {
    Map<String, JsonNode> versions = new TreeMap<>(SemanticVersions.ORDER);
    for (JsonNode document : documents) {
      for (JsonNode version : document.path("versions")) {
        String value = version.path("version").asText();
        if (!value.isEmpty()) {
          versions.putIfAbsent(value, version);
        }
      }
    }
    ObjectNode merged = TerraformResponses.object();
    ArrayNode list = merged.putArray("versions");
    versions.values().forEach(list::add);
    return merged;
  }

  static ObjectNode mergeMirrorIndexes(final List<JsonNode> documents) {
    Map<String, JsonNode> versions = new TreeMap<>(SemanticVersions.ORDER);
    for (JsonNode document : documents) {
      document.path("versions").fields().forEachRemaining(entry -> versions.putIfAbsent(entry.getKey(),
          entry.getValue()));
    }
    ObjectNode merged = TerraformResponses.object();
    ObjectNode list = merged.putObject("versions");
    versions.forEach(list::set);
    return merged;
  }

  static ObjectNode mergeMirrorVersions(final List<JsonNode> documents) {
    Map<String, JsonNode> archives = new LinkedHashMap<>();
    for (JsonNode document : documents) {
      document.path("archives").fields().forEachRemaining(entry -> archives.putIfAbsent(entry.getKey(),
          entry.getValue()));
    }
    ObjectNode merged = TerraformResponses.object();
    ObjectNode list = merged.putObject("archives");
    new TreeMap<>(archives).forEach(list::set);
    return merged;
  }
}
