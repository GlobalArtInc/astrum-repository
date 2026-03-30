package org.globalart.astrum.content.npm.internal.group;

import java.io.IOException;
import java.io.InputStream;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.npm.internal.NpmJsonUtils;
import org.globalart.astrum.repository.npm.internal.NpmPackageId;
import org.globalart.astrum.repository.npm.internal.NpmPaths;
import org.globalart.astrum.repository.npm.internal.NpmResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.npm.internal.NpmVersionComparator.extractNewestVersion;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

@Component
public class NpmGroupDistTagsHandler
    extends GroupHandler
{
  @Nonnull
  @Override
  protected Response doGet(
      @Nonnull final Context context,
      @Nonnull final DispatchedRepositories dispatched) throws Exception
  {
    NpmPackageId packageId = NpmPaths.packageId(context.getAttributes().require(TokenMatcher.State.class));
    List<Repository> members = context.getRepository().facet(GroupFacet.class).members();

    Map<String, Object> merged = new LinkedHashMap<>();
    boolean found = false;
    for (Response response : getAll(context, members, dispatched).values()) {
      Payload payload = response.getPayload();
      if (!response.getStatus().isSuccessful() || payload == null) {
        continue;
      }
      NestedAttributesMap distTags = read(payload);
      if (distTags == null) {
        continue;
      }
      found = true;
      distTags.backing().forEach((tag, version) -> merged.merge(tag, version,
          (current, candidate) -> extractNewestVersion.apply(current.toString(), candidate.toString())));
    }

    if (!found) {
      return NpmResponses.packageNotFound(packageId);
    }
    NestedAttributesMap result = new NestedAttributesMap("dist-tags", merged);
    return NpmResponses.ok(new BytesPayload(NpmJsonUtils.bytes(result), APPLICATION_JSON));
  }

  private NestedAttributesMap read(final Payload payload) {
    try (Payload closeable = payload; InputStream in = closeable.openInputStream()) {
      return NpmJsonUtils.parse(() -> in);
    }
    catch (IOException e) {
      log.debug("Unable to read dist-tags response", e);
      return null;
    }
  }
}
