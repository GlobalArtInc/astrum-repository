
package org.globalart.astrum.siesta;

import jakarta.ws.rs.client.WebTarget;
import jakarta.ws.rs.core.Response;

import org.globalart.astrum.rest.ExceptionMapperSupport;

import org.junit.jupiter.api.Test;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import static org.hamcrest.MatcherAssert.assertThat;
import static org.hamcrest.Matchers.equalTo;
import static org.hamcrest.Matchers.notNullValue;

/**
 * Tests error handling.
 */
class ErrorsIT
    extends SiestaTestSupport
{
  private static final Logger log = LoggerFactory.getLogger(ErrorsIT.class);

  @Test
  void errorResponseHasFaultId() {
    WebTarget target = client().target(url("errors/406"));
    Response response = target.request().get(Response.class);
    log.info("Status: {}", response.getStatusInfo());

    assertThat(response.getStatusInfo().getStatusCode(), equalTo(406));
    String faultId = response.getHeaderString(ExceptionMapperSupport.X_SIESTA_FAULT_ID);
    log.info("Fault ID: {}", faultId);
    assertThat(faultId, notNullValue());
  }
}
