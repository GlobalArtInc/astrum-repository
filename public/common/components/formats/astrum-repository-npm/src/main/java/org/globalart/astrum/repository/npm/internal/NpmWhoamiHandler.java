
package org.globalart.astrum.repository.npm.internal;

import java.util.Collections;
import java.util.Map;

import org.springframework.beans.factory.annotation.Autowired;
import org.springframework.stereotype.Component;

import javax.annotation.Nonnull;

import org.globalart.astrum.repository.http.HttpResponses;
import org.globalart.astrum.repository.view.Context;
import org.globalart.astrum.repository.view.Handler;
import org.globalart.astrum.repository.view.Response;
import org.globalart.astrum.repository.view.payloads.StringPayload;
import org.globalart.astrum.security.SecuritySystem;
import org.globalart.astrum.security.user.User;

import com.fasterxml.jackson.databind.ObjectMapper;

import static com.google.common.base.Preconditions.checkNotNull;
import static org.globalart.astrum.repository.view.ContentTypes.APPLICATION_JSON;

/**
 * @since 3.20
 */
@Component
public class NpmWhoamiHandler
    implements Handler
{
  private final SecuritySystem securitySystem;

  private final ObjectMapper objectMapper = new ObjectMapper();

  @Autowired
  public NpmWhoamiHandler(final SecuritySystem securitySystem) {
    this.securitySystem = checkNotNull(securitySystem);
  }

  @Nonnull
  @Override
  public Response handle(@Nonnull final Context context) throws Exception
  {
    User user = securitySystem.currentUser();

    Map<String, String> username = Collections.singletonMap("username", user == null ? "anonymous" : user.getUserId());

    StringPayload payload = new StringPayload(objectMapper.writeValueAsString(username), APPLICATION_JSON);

    return HttpResponses.ok(payload);
  }
}
