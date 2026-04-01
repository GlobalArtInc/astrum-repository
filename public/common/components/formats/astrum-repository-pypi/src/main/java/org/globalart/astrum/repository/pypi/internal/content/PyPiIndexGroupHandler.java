
package org.globalart.astrum.repository.pypi.internal.content;

import java.util.Map;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.template.TemplateHelper;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.pypi.internal.AssetKind;
import org.globalart.astrum.repository.pypi.internal.PyPiFormat;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.beans.factory.annotation.Qualifier;
import org.springframework.stereotype.Component;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.pypi.internal.AssetKind.ROOT_INDEX;
import static org.globalart.astrum.repository.pypi.internal.PyPiGroupUtils.lazyMergeResult;
import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.name;
import static org.globalart.astrum.repository.pypi.internal.PyPiPathUtils.normalizeName;
import static org.globalart.astrum.repository.pypi.internal.content.ContentPypiPathUtils.indexPath;

@Component
@Qualifier(PyPiFormat.NAME)
public class PyPiIndexGroupHandler
    extends GroupHandler
{
  private final TemplateHelper templateHelper;

  @Autowired
  public PyPiIndexGroupHandler(final TemplateHelper templateHelper) {
    this.templateHelper = checkNotNull(templateHelper);
  }

  @Override
  protected Response doGet(@Nonnull final Context context, @Nonnull final DispatchedRepositories dispatched)
      throws Exception
  {
    AssetKind assetKind = context.getAttributes().require(AssetKind.class);
    String name;
    String path;
    if (ROOT_INDEX == assetKind) {
      name = null;
      path = indexPath();
    }
    else {
      name = normalizeName(name(context.getAttributes().require(TokenMatcher.State.class)));
      path = indexPath(name);
    }

    PyPiGroupFacet groupFacet = context.getRepository().facet(PyPiGroupFacet.class);
    Content content = groupFacet.getFromCache(path);
    Map<Repository, Response> memberResponses = getAll(context, groupFacet.members(), dispatched);

    if (groupFacet.isStale(path, content, memberResponses)) {
      if (memberResponses.values().stream().noneMatch(response -> response.getStatus().isSuccessful())) {
        return HttpResponses.notFound();
      }
      return HttpResponses.ok(
          groupFacet.buildIndex(path, lazyMergeResult(name, assetKind, memberResponses, templateHelper)));
    }
    return HttpResponses.ok(content);
  }
}
