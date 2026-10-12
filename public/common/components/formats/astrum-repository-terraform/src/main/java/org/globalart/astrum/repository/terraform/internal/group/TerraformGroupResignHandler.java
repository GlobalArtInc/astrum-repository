package org.globalart.astrum.repository.terraform.internal.group;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.terraform.TerraformContentFacet;
import org.globalart.astrum.repository.terraform.TerraformRequest;
import org.globalart.astrum.repository.terraform.internal.DownloadTokens;
import org.globalart.astrum.repository.terraform.internal.TerraformResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;

import com.fasterxml.jackson.databind.JsonNode;
import com.fasterxml.jackson.databind.node.ObjectNode;
import org.springframework.stereotype.Component;

import static org.globalart.astrum.repository.terraform.internal.TerraformResponses.X_TERRAFORM_GET;

/**
 * Members sign the download URLs of their answers for themselves; the group signs them again for itself, as
 * Terraform fetches them through the group.
 */
@Component
public class TerraformGroupResignHandler
    implements Handler
{
  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception {
    Response response = context.proceed();
    if (!response.getStatus().isSuccessful()) {
      return response;
    }
    String documentPath = context.getRequest().getPath();
    DownloadTokens tokens = DownloadTokens.of(context.getRepository().facet(TerraformContentFacet.class));
    switch (TerraformRequest.of(context).endpoint()) {
      case MODULE_DOWNLOAD -> {
        String location = response.getHeaders().get(X_TERRAFORM_GET);
        return location == null ? response : TerraformResponses.location(tokens.resign(location, documentPath));
      }
      case PROVIDER_DOWNLOAD -> {
        byte[] body = TerraformResponses.body(response);
        if (body == null) {
          return response;
        }
        JsonNode document = TerraformResponses.read(body);
        if (!(document instanceof ObjectNode object)) {
          return TerraformResponses.json(document);
        }
        for (String field : new String[]{"download_url", "shasums_url", "shasums_signature_url"}) {
          if (object.hasNonNull(field)) {
            object.put(field, tokens.resign(object.get(field).asText(), documentPath));
          }
        }
        return TerraformResponses.json(object);
      }
      default -> {
        return response;
      }
    }
  }
}
