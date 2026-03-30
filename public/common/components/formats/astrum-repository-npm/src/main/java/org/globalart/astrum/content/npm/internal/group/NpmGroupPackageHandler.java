package org.globalart.astrum.content.npm.internal.group;

import java.io.IOException;
import java.io.InputStream;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Map;

import javax.annotation.Nonnull;

import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.common.io.InputStreamSupplier;
import org.globalart.astrum.repository.Repository;
import org.globalart.astrum.repository.group.GroupFacet;
import org.globalart.astrum.repository.group.GroupHandler;
import org.globalart.astrum.repository.npm.internal.NpmJsonUtils;
import org.globalart.astrum.repository.npm.internal.NpmMergeObjectMapper;
import org.globalart.astrum.repository.npm.internal.NpmMetadataUtils;
import org.globalart.astrum.repository.npm.internal.NpmPackageId;
import org.globalart.astrum.repository.npm.internal.NpmPaths;
import org.globalart.astrum.repository.npm.internal.NpmResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.matchers.token.TokenMatcher;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import org.springframework.beans.factory.annotation.Value;
import org.springframework.stereotype.Component;

import static org.apache.commons.lang3.StringUtils.isEmpty;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

@Component
public class NpmGroupPackageHandler
    extends GroupHandler
{
  private final boolean mergeMetadata;

  public NpmGroupPackageHandler(@Value("${astrum.npm.mergeGroupMetadata:true}") final boolean mergeMetadata) {
    this.mergeMetadata = mergeMetadata;
  }

  @Nonnull
  @Override
  protected Response doGet(
      @Nonnull final Context context,
      @Nonnull final DispatchedRepositories dispatched) throws Exception
  {
    NpmPackageId packageId = NpmPaths.packageId(context.getAttributes().require(TokenMatcher.State.class));
    List<Repository> members = context.getRepository().facet(GroupFacet.class).members();
    Map<Repository, Response> responses = getAll(context, members, dispatched);

    List<Payload> payloads = new ArrayList<>();
    try {
      for (Response response : responses.values()) {
        if (response.getStatus().isSuccessful() && response.getPayload() != null) {
          payloads.add(response.getPayload());
        }
      }
      if (payloads.isEmpty()) {
        return NpmResponses.packageNotFound(packageId);
      }

      NestedAttributesMap packageRoot = merge(packageId, payloads);
      NpmMetadataUtils.rewriteTarballUrl(context.getRepository().getName(), packageRoot);
      return NpmResponses.ok(new BytesPayload(NpmJsonUtils.bytes(packageRoot), APPLICATION_JSON));
    }
    finally {
      for (Response response : responses.values()) {
        closeQuietly(response.getPayload());
      }
    }
  }

  private NestedAttributesMap merge(final NpmPackageId packageId, final List<Payload> payloads) throws IOException {
    if (payloads.size() == 1 || (!mergeMetadata && isEmpty(packageId.scope()))) {
      try (InputStream in = payloads.get(0).openInputStream()) {
        NestedAttributesMap root = new NpmMergeObjectMapper().read(in);
        root.remove(NpmMetadataUtils.META_REV);
        return root;
      }
    }

    List<InputStreamSupplier> streams = new ArrayList<>();
    for (Payload payload : payloads) {
      streams.add(payload::openInputStream);
    }
    Collections.reverse(streams);
    return new NpmMergeObjectMapper().merge(streams);
  }

  private void closeQuietly(final AutoCloseable closeable) {
    if (closeable == null) {
      return;
    }
    try {
      closeable.close();
    }
    catch (Exception e) {
      log.debug("Unable to close {}", closeable, e);
    }
  }
}
