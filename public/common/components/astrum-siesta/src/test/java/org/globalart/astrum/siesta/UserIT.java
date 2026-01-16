
package org.globalart.astrum.siesta;

import java.util.UUID;

import jakarta.ws.rs.client.Entity;
import jakarta.ws.rs.client.WebTarget;
import jakarta.ws.rs.core.MediaType;
import jakarta.ws.rs.core.Response;
import jakarta.ws.rs.core.Response.Status.Family;

import org.junit.jupiter.api.Test;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static jakarta.ws.rs.core.MediaType.APPLICATION_JSON_TYPE;
import static jakarta.ws.rs.core.MediaType.APPLICATION_XML_TYPE;
import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.is;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Tests related to happy paths for a resource.
 */
class UserIT
    extends SiestaTestSupport
{
  private static final Logger log = LoggerFactory.getLogger(UserIT.class);

  private void put_happyPath(final MediaType mediaType) {
    UserXO sent = new UserXO().withName(UUID.randomUUID().toString());

    WebTarget target = client().target(url("user"));
    Response response = target.request()
        .accept(mediaType)
        .put(Entity.entity(sent, mediaType), Response.class);
    log.info("Status: {}", response.getStatusInfo());

    assertThat(response.getStatusInfo().getFamily(), equalTo(Family.SUCCESSFUL));

    UserXO received = response.readEntity(UserXO.class);
    assertThat(received, is(notNullValue()));
    assertThat(received.getName(), is(equalTo(sent.getName())));
  }

  @Test
  void put_happyPath_XML() {
    put_happyPath(APPLICATION_XML_TYPE);
  }

  @Test
  void put_happyPath_JSON() {
    put_happyPath(APPLICATION_JSON_TYPE);
  }
}
