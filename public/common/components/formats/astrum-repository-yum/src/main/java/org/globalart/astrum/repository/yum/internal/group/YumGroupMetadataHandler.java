package org.globalart.astrum.repository.yum.internal.group;

import java.io.InputStream;
import java.util.Optional;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Content;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.BytesPayload;
import org.globalart.astrum.repository.yum.internal.YumPaths;
import org.globalart.astrum.repository.yum.internal.YumSigningFacet;
import org.globalart.astrum.repository.yum.internal.group.YumGroupFacet.GroupRepomd;

import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.yum.internal.YumPaths.requestPath;

@Component
public class YumGroupMetadataHandler
    extends GroupHandler
{
  @Override
  protected Response doGet(@Nonnull final Context context, @Nonnull final DispatchedRepositories dispatched)
      throws Exception
  {
    String path = requestPath(context);
    String name = YumPaths.fileName(path);
    YumGroupFacet group = context.getRepository().facet(YumGroupFacet.class);
    YumSigningFacet signing = context.getRepository().facet(YumSigningFacet.class);
    if (YumPaths.PUBLIC_KEY.equals(name) && signing.isConfigured()) {
      return signing.publicKey().map(HttpResponses::ok).orElseGet(HttpResponses::notFound);
    }
    if (YumPaths.SIGNATURE.equals(name)) {
      return signature(context, group, signing, path);
    }
    Optional<Content> merged = group.getMetadata(path);
    if (merged.isPresent()) {
      return HttpResponses.ok(merged.get());
    }
    return super.doGet(context, dispatched);
  }

  private Response signature(
      final Context context,
      final YumGroupFacet group,
      final YumSigningFacet signing,
      final String path) throws Exception
  {
    String base = YumPaths.baseOfMetadata(path).orElse("");
    Optional<GroupRepomd> repomd = group.repomd(context, base);
    if (repomd.isEmpty()) {
      return notFoundResponse(context);
    }
    if (signing.isConfigured()) {
      byte[] bytes;
      try (InputStream in = repomd.get().content().openInputStream()) {
        bytes = in.readAllBytes();
      }
      return signing.sign(bytes)
          .map(signature -> HttpResponses.ok(new BytesPayload(signature, YumSigningFacet.SIGNATURE_CONTENT_TYPE)))
          .orElseGet(HttpResponses::notFound);
    }
    if (repomd.get().passedThrough() != null) {
      Response response = YumGroupFacet.dispatch(repomd.get().passedThrough(), context, path,
          new DispatchedRepositories());
      return response.getStatus().isSuccessful() ? response : notFoundResponse(context);
    }
    return notFoundResponse(context);
  }
}
