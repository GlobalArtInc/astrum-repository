package org.globalart.astrum.repository.conda.internal.group;

import java.io.InputStream;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.conda.internal.metadata.Repodata;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.http.HttpStatus;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Request;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.ViewFacet;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher.State;

import com.github.luben.zstd.ZstdInputStream;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.conda.util.CondaPathUtils.REPODATA_JSON_ZST;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.matcherState;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.repodataPath;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.subdir;
import static org.globalart.astrum.repository.conda.util.CondaPathUtils.subdirPath;
import static org.globalart.astrum.repository.http.HttpMethods.GET;

@Component
public class CondaRepodataGroupHandler
    extends GroupHandler
{
  @Override
  protected Response doGet(@Nonnull final Context context, @Nonnull final DispatchedRepositories dispatched)
      throws Exception
  {
    State state = matcherState(context);
    String subdir = subdir(state);
    String path = repodataPath(subdirPath(state));
    CondaGroupFacet groupFacet = context.getRepository().facet(CondaGroupFacet.class);

    Map<Repository, Response> responses = new LinkedHashMap<>();
    Map<Repository, Boolean> compressed = new LinkedHashMap<>();
    collectMemberRepodata(context, groupFacet, subdirPath(state), dispatched, responses, compressed);

    Content cached = groupFacet.getFromCache(path);
    if (!groupFacet.isStale(cached, responses)) {
      return HttpResponses.ok(cached);
    }
    if (responses.isEmpty()) {
      return HttpResponses.notFound();
    }

    List<Repodata.Source> sources = new ArrayList<>();
    for (Map.Entry<Repository, Response> entry : responses.entrySet()) {
      Content content = (Content) entry.getValue().getPayload();
      boolean zstd = compressed.get(entry.getKey());
      sources.add(() -> {
        InputStream in = content.openInputStream();
        return zstd ? new ZstdInputStream(in) : in;
      });
    }
    return HttpResponses.ok(groupFacet.buildRepodata(path, subdir, sources));
  }

  private void collectMemberRepodata(
      final Context context,
      final CondaGroupFacet groupFacet,
      final String subdirPath,
      final DispatchedRepositories dispatched,
      final Map<Repository, Response> responses,
      final Map<Repository, Boolean> compressed) throws Exception
  {
    Request zstRequest = memberRequest(subdirPath + "/" + REPODATA_JSON_ZST, dispatched);
    Request jsonRequest = memberRequest(repodataPath(subdirPath), dispatched);
    for (Repository member : groupFacet.members()) {
      if (member.getConfiguration() != null && !member.getConfiguration().isOnline()) {
        continue;
      }
      if (dispatched.contains(member)) {
        continue;
      }
      dispatched.add(member);
      ViewFacet view = member.facet(ViewFacet.class);
      Response response = view.dispatch(zstRequest, context);
      if (isRepodata(response)) {
        responses.put(member, response);
        compressed.put(member, true);
        continue;
      }
      response = view.dispatch(jsonRequest, context);
      if (isRepodata(response)) {
        responses.put(member, response);
        compressed.put(member, false);
      }
    }
  }

  private static Request memberRequest(final String path, final DispatchedRepositories dispatched) {
    Request.Builder builder = new Request.Builder().action(GET).path(path);
    builder.attributes().set(DispatchedRepositories.class, dispatched);
    return builder.build();
  }

  private static boolean isRepodata(final Response response) {
    return response.getStatus().getCode() == HttpStatus.OK && response.getPayload() instanceof Content;
  }
}
