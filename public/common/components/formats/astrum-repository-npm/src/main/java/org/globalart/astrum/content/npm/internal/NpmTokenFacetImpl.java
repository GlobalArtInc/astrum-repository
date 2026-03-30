
package org.globalart.astrum.content.npm.internal;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;
import org.springframework.context.annotation.Scope;
import org.springframework.beans.factory.config.ConfigurableBeanFactory;

import java.io.IOException;
import java.io.InputStream;


import org.globalart.astrum.common.collect.NestedAttributesMap;
import org.globalart.astrum.repository.FacetSupport;
import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.npm.internal.NpmJsonUtils;
import org.globalart.astrum.repository.npm.internal.NpmResponses;
import org.globalart.astrum.repository.npm.internal.NpmTokenFacet;
import org.globalart.astrum.repository.npm.internal.security.NpmTokenManager;
import org.globalart.astrum.repository.view.ContentTypes;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Payload;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.BytesPayload;

import com.google.common.collect.Maps;

import static com.google.common.base.Preconditions.checkNotNull;

/**
 * Implementation of {@link NpmTokenFacet}.
 *
 * @since 3.0
 */
@Component
@Scope(ConfigurableBeanFactory.SCOPE_PROTOTYPE)
public class NpmTokenFacetImpl
    extends FacetSupport
    implements NpmTokenFacet
{
  private final NpmTokenManager npmTokenManager;

  @Autowired
  public NpmTokenFacetImpl(final NpmTokenManager npmTokenManager) {
    this.npmTokenManager = checkNotNull(npmTokenManager);
  }

  @Override
  public Response login(final Context context) {
    final Payload payload = context.getRequest().getPayload();
    if (payload == null) {
      return NpmResponses.badRequest("Missing body");
    }
    try (InputStream in = payload.openInputStream()) {
      NestedAttributesMap request = NpmJsonUtils.parse(() -> in);
      String token = npmTokenManager.login(request.get("name", String.class), request.get("password", String.class));
      if (null != token) {
        NestedAttributesMap response = new NestedAttributesMap("response", Maps.newHashMap());
        response.set("ok", Boolean.TRUE.toString());
        response.set("rev", "_we_dont_use_revs_any_more");
        response.set("id", "org.couchdb.user:undefined");
        response.set("token", token);
        return HttpResponses.created(new BytesPayload(NpmJsonUtils.bytes(response), ContentTypes.APPLICATION_JSON));
      }
      else {
        return NpmResponses.badCredentials("Bad username or password");
      }
    }
    catch (IOException e) {
      throw new RuntimeException(e);
    }
  }

  @Override
  public Response logout(final Context context) {
    if (npmTokenManager.logout()) {
      NestedAttributesMap response = new NestedAttributesMap("response", Maps.newHashMap());
      response.set("ok", Boolean.TRUE.toString());
      return NpmResponses.ok(new BytesPayload(NpmJsonUtils.bytes(response), ContentTypes.APPLICATION_JSON));
    }
    else {
      return NpmResponses.notFound("Token not found");
    }
  }
}
