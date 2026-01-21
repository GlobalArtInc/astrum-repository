
package org.globalart.astrum.api.rest.common.status;

import org.junit.Before;
import org.junit.Test;

import jakarta.ws.rs.core.Response;

import static org.junit.Assert.assertEquals;

public class StatusResourceTest
{
  private StatusResource statusResource;

  @Before
  public void setUp() {
    statusResource = new StatusResource();
  }

  @Test
  public void isAvailableAlwaysReturnsOk() {
    Response response = statusResource.isAvailable();
    assertEquals(200, response.getStatus());
  }

  @Test
  public void isWritableAlwaysReturnsOk() {
    Response response = statusResource.isWritable();
    assertEquals(200, response.getStatus());
  }
}
